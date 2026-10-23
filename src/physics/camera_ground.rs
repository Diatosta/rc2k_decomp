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
use crate::framework::Regs;
use crate::physics::flow::{g, p};
use crate::physics::reset_parts::{COUNT_PROBE, game_part};
use crate::race::car_starts::Part;

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

/// `dispatch_carry` without its hook (Rust callers; a refusal is the caller's).
pub fn carry_entry(r: &mut Regs, rust: Check) -> Result<(), Refused> {
    let left = unsafe { rust(r, &mut game_part) }?;
    let esp = r.esp;
    *r = left;
    r.esp = esp;
    Ok(())
}
