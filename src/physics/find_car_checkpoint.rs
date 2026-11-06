//! `physics::find_car_checkpoint` (0x44139D): the work car's checkpoint looked for near
//! the last one. The car's x and z into `dword_639824` / `dword_63982C`; with no
//! checkpoints eax 0. Else `race::nearest_checkpoint_in` over `g_car` +0x98 - 1 ..
//! +0x98 + 1 (ecx, edx); its distance (edi) through `dword_62400C`, / 65536 into
//! `g_nearestCheckpointDistance`; above 0xA0000 (unsigned) `race::nearest_checkpoint`
//! over them all (eax), else eax the nearest (ebp).
//!
//! In:  nothing (the registers are passed on)
//! Out: eax the checkpoint; the rest as the calls leave them; no flags read (the caller
//!      stores eax)
//!
//! Caller: `sub_44157B`. Refuses what the two calls refuse.

use crate::errors::Refused;
use crate::math::x87_run::{ext_div2, int_80};
use crate::race::owner::Race;
use crate::scene::owner::scene;
use rc2k_core::physics::progress;

/// The Rust version: the car's checkpoint (eax): the nearest within the window about the
/// one it had (`race::nearest_checkpoint_in`), or, when that is too far, of them all
/// (`race::nearest_checkpoint`). The other registers it leaves are read by none of its
/// callers.
// Converted: `rc2k_core::physics::progress::{near_window, search_all}`.
pub fn find_car_checkpoint(race: &Race) -> Result<u32, Refused> {
    let race_owner = race;
    race_owner
        .physics
        .ground
        .ground_x
        .set(crate::race::owner::race().shared.car.position[0].get());
    race_owner
        .physics
        .ground
        .ground_z
        .set(crate::race::owner::race().shared.car.position[2].get());
    if crate::race::owner::race()
        .shared
        .vars
        .checkpoint_count
        .get()
        == 0
    {
        return Ok(0);
    }
    let (from, to) = progress::near_window(crate::race::owner::race().shared.car.checkpoint.get());
    let near = crate::race::start_place::nearest_checkpoint_in(race, from, to)?;
    scene().drawing.scene.scratch.set(near.distance);
    // g_nearestCheckpointDistance
    crate::race::owner::race()
        .shared
        .physics_tables
        .nearest_checkpoint_distance
        .set_bits(ext_div2(
            int_80(near.distance as i32, false),
            race_owner.course.numbers.k65536.bits(),
            0x3F80_0000,
        ));
    if progress::search_all(near.distance) {
        return crate::race::start_place::nearest_checkpoint(race);
    }
    Ok(near.checkpoint)
}
