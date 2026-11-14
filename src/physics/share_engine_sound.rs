//! For the current car's sound object (in the race's sound objects): when `contact_depth` is above
//! the object's stored depth (not equal, not below, not unordered), the work car's contact place
//! and size go into the object; always `steer_trim` goes into its steering field and its update
//! flag is 1. So it is not the engine's figures but the car's contact (its place and size, kept in
//! `knock_in` while it is the deepest so far) and the steering pull (from `steer_trim`) that the
//! sounds read.
//!
//! Caller: `physics::car_physics_step`, every physics step. Refuses (before anything) a null sound
//! object.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use rc2k_core::physics::engine::shares_figures;

/// A compare of floats only: the same at any precision.
// Core: `rc2k_core::physics::engine::shares_figures`.
pub fn share_engine_sound(
    car: &Car,
    sound: &crate::sound::owner::Sound,
    work: &crate::race::work_store::RaceWork,
) -> Result<(), Refused> {
    let so = crate::sound::store::current_object(sound, work);
    if shares_figures(car.contact_depth.get(), so.knock_in[3].get()) {
        for (k, figure) in car
            .contact_point
            .iter()
            .chain([&car.contact_depth])
            .enumerate()
        {
            so.knock_in[k].set(figure.get());
        }
    }
    so.steer_trim_copy.set(car.steer_trim.get());
    so.new_revs.set(1);
    Ok(())
}
