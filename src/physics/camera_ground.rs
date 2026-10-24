//! The camera's ground checks (stream B, 2026-10-18): `physics::camera_low_switch`
//! (0x438BF4), `physics::camera_over_ground` (0x438CEC), `physics::camera_clear_of_ground`
//! (0x438DD2). Each probes the ground under a point through `physics::count_ground_probe`
//! (called directly), starting from the face it found there last
//! (`unk_71AEC8` .., into `dword_5CCC78`) and keeping the face found (`dword_5CCC74`).
//! What the camera variables (`unk_71A65C` .. `unk_71A6A4`) are is a reading: the mode,
//! the place (`unk_71A67C` ..), the turns (`unk_71A688` ..).
//!
//! This file: the shared steps.

use crate::errors::Refused;
use crate::physics::flow::{g, p};

/// `g_groundFound`.
pub const FOUND: usize = 0x639830;

/// One probe at (x, z) from the face kept at `slot`; true when found (the face found
/// kept there).
pub fn probe_at(x: u32, z: u32, slot: usize) -> Result<bool, Refused> {
    p(0x5CCC78, g(slot));
    p(0x639824, x);
    p(0x63982C, z);
    unsafe { crate::physics::count_ground_probe::count_ground_probe() }?;
    if g(FOUND) == 0 {
        return Ok(false);
    }
    p(slot, g(0x5CCC74));
    Ok(true)
}
