//! The camera's ground checks (stream B, 2026-10-18): `physics::camera_low_switch`
//! (0x438BF4), `physics::camera_over_ground` (0x438CEC), `physics::camera_clear_of_ground`
//! (0x438DD2). Each probes the ground under a point through `physics::count_ground_probe`
//! (called directly), starting from the face it found there last
//! (`CarBlock::camera_ground_faces`, into `dword_5CCC78`) and keeping the face found (`dword_5CCC74`).
//! What the camera variables (`unk_71A65C` .. `unk_71A6A4`) are is a reading: the mode,
//! the place (`unk_71A67C` ..), the turns (`unk_71A688` ..).
//!
//! This file: the shared steps.

use crate::errors::Refused;
use crate::race::owner::Race;

/// One probe at (x, z) from the face kept at `slot`; true when found (the face found
/// kept there).
pub fn probe_at(
    race: &Race,
    x: u32,
    z: u32,
    slot: &crate::session::Dword,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    race.course.course.ground_face_last.set(slot.get());
    race.physics.ground.ground_x.set(x);
    race.physics.ground.ground_z.set(z);
    { crate::physics::count_ground_probe::count_ground_probe(race, scene) }?;
    if race.physics.ground.ground_found.get() == 0 {
        return Ok(false);
    }
    slot.set(race.course.course.ground_slot.get());
    Ok(true)
}
