//! For the current car's sound object (in the race's sound objects): when `contact_depth` is above
//! the object's stored depth (not equal, not below, not unordered), the work car's contact place
//! and size go into the object; always `steer_trim` goes into its steering field and its update
//! flag is 1. So it is not the engine's figures but the car's contact (its place and size, kept in
//! `knock_in` while it is the deepest so far) and the steering pull (from `steer_trim`) that the
//! sounds read.
//!
//! Refuses (before anything) a null sound object.

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
    if shares_figures(car.contact_depth, so.borrow().knock_in[3]) {
        let figures = {
            let [x, y, z] = car.contact_point;
            [x, y, z, car.contact_depth]
        };
        for (k, figure) in figures.into_iter().enumerate() {
            so.borrow_mut().knock_in[k] = figure;
        }
    }
    so.borrow_mut().steer_trim_copy = car.steer_trim;
    so.borrow_mut().new_revs = 1;
    Ok(())
}
