//! The camera's ground checks (stream B, 2026-10-18): `physics::camera_low_switch`
//! (0x438BF4), `physics::camera_over_ground` (0x438CEC), `physics::camera_clear_of_ground`
//! (0x438DD2). Each probes the ground under a point through `physics::count_ground_probe`
//! (place 19 of `physics/reset_parts.rs`), starting from the face it found there last
//! (`unk_71AEC8` .., into `dword_5CCC78`) and keeping the face found (`dword_5CCC74`).
//! What the camera variables (`unk_71A65C` .. `unk_71A6A4`) are is a reading: the mode,
//! the place (`unk_71A67C` ..), the turns (`unk_71A688` ..).
//!
//! This file: the shared steps, the dispatch (keeping the carry) and the tests' frame.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, shadow};
use crate::physics::flow::{a, g, p};
use crate::physics::reset_parts::{COUNT_PROBE, game_part, regions, start_case, test_part};
use crate::race::car_starts::{Part, take_called};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture};

/// `g_groundFound`.
pub const FOUND: usize = 0x639830;

/// One probe at (x, z) from the face kept at `slot`; true when found (the face found
/// kept there).
pub fn probe_at(r: &mut Regs, part: Part, x: u32, z: u32, slot: usize) -> Result<bool, Refused> {
    p(0x5CCC78, g(slot));
    p(0x639824, x);
    p(0x63982C, z);
    part(COUNT_PROBE, r)?;
    if g(FOUND) == 0 {
        return Ok(false);
    }
    p(slot, g(0x5CCC74));
    Ok(true)
}

/// The Rust version's form here: the registers and the carry left.
pub type Check = unsafe fn(&Regs, Part) -> Result<Regs, Refused>;

/// The dispatch: the original alone in shadow mode (its probes are `find_ground`'s,
/// whose own shadow runs the original alone), else the Rust version, its carry kept.
pub fn dispatch_carry(
    hook: &RawHook,
    r: &mut Regs,
    call_original: unsafe extern "C" fn(*mut Regs),
    rust: Check,
) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { rust(r, &mut game_part) } {
        Ok(left) => {
            let esp = r.esp;
            *r = left;
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

/// `dispatch_carry` without its hook (Rust callers; a refusal is the caller's).
pub fn carry_entry(r: &mut Regs, rust: Check) -> Result<(), Refused> {
    let left = unsafe { rust(r, &mut game_part) }?;
    let esp = r.esp;
    *r = left;
    r.esp = esp;
    Ok(())
}

/// What the tests set and compare besides `reset_parts::regions()`.
pub fn test_regions() -> Vec<Region> {
    vec![
        (a(0x5CCC74), 8),
        (a(0x639824), 0x10),
        (a(0x620470), 4),
        (a(0x608C3C), 12),
        (a(0x71A65C), 0x48),
        (a(0x71AEC8), 12),
        (a(0x71A0A0), 4),
        (a(0x719FE0), 4),
        crate::math::turn_scratch_xz::scratch(),
    ]
}

/// A copy of a check whose probes go to the stand-in (place 19; `g_groundFound` stays as
/// the case set it), against the Rust version with the same: registers, carry, the
/// stand-in's calls and memory compared.
pub fn both_carry(
    t: &mut SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    entry: Regs,
    fill: &dyn Fn(),
    rust: Check,
) {
    let regs = |r: &Regs| (r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp, r.carry());
    let mut all = regions();
    all.extend(test_regions());
    fill();
    start_case(0);
    let mut ro = entry;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (Some(regs(&ro)), take_called(), capture(&all));
    fill();
    start_case(0);
    let mine = unsafe { rust(&entry, &mut test_part) };
    let mine = (mine.as_ref().ok().map(regs), take_called(), capture(&all));
    t.check(theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}
