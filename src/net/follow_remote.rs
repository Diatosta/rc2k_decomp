//! `net::follow_remote` (0x464883): a remote car's step in a network race: while it has
//! steps of a glide left (the current car's block +0xFCC, set to 32 by
//! `net::apply_message`), one less, and a 32nd of the way (+0xFC0 .. +0xFC8) added to its
//! place (+0x134 .. +0x13C).
//!
//! In:  nothing
//! Out: eax +0xFC8 after a glide step, else kept; the rest kept
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Calls nothing. Shadow mode runs the
//! original alone. Self-test: a copy whose switch and car block are the test's, against
//! the Rust version: the network off, no steps left, steps left.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{CAR, ON, Part, a, dispatch_net, net_entry};
use crate::system::api_stand_ins::copy_with;
use rc2k_core::net::race;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::follow_remote", rva(0x464883), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on the switch at `on` and the block at `car`.
pub fn follow_remote_with(r: &mut Regs, on: u32, car: u32, _part: Part) -> Result<(), Refused> {
    let rd = |at: u32| unsafe { (at as *const u32).read_volatile() };
    let wr = |at: u32, v: u32| unsafe { (at as *mut u32).write_volatile(v) };
    // Converted: `rc2k_core::net::race::glide` (the block read only with the race on).
    let three = |at: u32| [rd(car + at), rd(car + at + 4), rd(car + at + 8)];
    if rd(on) == 0 {
        return Ok(());
    }
    let steps = rd(car + race::GLIDE_STEPS);
    if steps == 0 {
        return Ok(());
    }
    let step = three(race::WANTED);
    if let Some((left, place)) = race::glide(true, steps, three(race::PLACE), step) {
        wr(car + race::GLIDE_STEPS, left);
        for (k, v) in (0..).zip(place) {
            wr(car + race::PLACE + 4 * k, v);
        }
        r.eax = step[2];
    }
    Ok(())
}

/// The Rust version.
pub fn follow_remote(r: &mut Regs, part: Part) -> Result<(), Refused> {
    follow_remote_with(r, a(ON), a(CAR), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, follow_remote);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, follow_remote)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut block = vec![0u32; 0x1010 / 4];
    let mut on = 0u32;
    let (car, on_at) = (block.as_mut_ptr() as u32, &raw mut on as u32);
    // cmp dword_52A910, 0
    let entry = [0x83, 0x3D, 0x10, 0xA9, 0x52, 0x00, 0x00];
    let copy = match copy_with(
        0x464883,
        0x3A,
        &entry,
        &[],
        &[],
        &[],
        &[(ON, 4, on_at), (CAR, 0x1008, car)],
        9,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(car, 0x1010usize), (on_at, 4)];
    for (switch, left) in [(0u32, 5u32), (1, 0), (1, 1), (1, 32), (2, 0xFFFF_FFFF)] {
        let fill = || unsafe {
            for n in 0..0x1010 / 4 {
                ((car + 4 * n) as *mut u32).write_volatile(n.wrapping_mul(0x9E37_79B9));
            }
            ((car + 0xFCC) as *mut u32).write_volatile(left);
            (on_at as *mut u32).write_volatile(switch);
        };
        compare(
            &mut t,
            &format!("switch {switch}, steps left {left:#x}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            &all,
            &|r, part| follow_remote_with(r, on_at, car, part),
            false,
        );
    }
    let _ = (&mut block, &mut on);
    t.finish()
}
