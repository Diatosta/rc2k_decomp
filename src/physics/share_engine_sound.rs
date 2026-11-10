//! `physics::share_engine_sound` (0x45D294): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`): when `flt_71A208` is above its +0xD0 (`fcomp`:
//! not equal, not below, not unordered), the work car's 0x71A1FC .. 0x71A208 into its
//! +0xC4 .. +0xD0; always `dword_71A1E0` into +0xD4 and +0x10 = 1. By the look of it the
//! engine's figures for the sounds (a reading).
//!
//! Returns: the sound object
//!
//! Caller: `physics::car_physics_step` (0x43B99F), every physics step. Refuses (before anything) a
//! null sound object.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::physics::engine::shares_figures;

/// A compare of floats only: the same at any precision.
// Converted: `rc2k_core::physics::engine::shares_figures`.
pub fn share_engine_sound(race: &Race) -> Result<(), Refused> {
    let so = crate::sound::store::current_object();
    if shares_figures(
        race.shared.car.contact_depth.get(),
        f32::from_bits(so.knock_in[3].get()),
    ) {
        for (k, figure) in race
            .shared
            .car
            .contact_point
            .iter()
            .chain([&race.shared.car.contact_depth])
            .enumerate()
        {
            so.knock_in[k].set(figure.bits());
        }
    }
    so.knock_d4.set(race.shared.car.steer_trim.get());
    so.new_revs.set(1);
    Ok(())
}
