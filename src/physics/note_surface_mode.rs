//! `physics::note_surface_mode` (0x43F180): `camera_mode_is_2_or_3` = 1 when `camera_view` is 2
//! or 3, else 0. What the mode is is not known.
//!
//! Caller: `physics::feel_ground` (0x43F1A8), first.

use crate::physics::car_store::Car;
use crate::session::Cell32;
use rc2k_core::physics::ground::surface_flag;

/// On the variables' cells `v`.
// Converted: `rc2k_core::physics::ground::surface_flag`.
pub fn note_at(v: &[&dyn Cell32]) {
    v[1].set_bits(surface_flag(v[0].bits()));
}

fn game(car: &Car) -> Vec<&dyn Cell32> {
    vec![&car.camera_view as &dyn Cell32, &car.camera_mode_is_2_or_3]
}

pub fn note_surface_mode(car: &Car) {
    note_at(&game(car));
}
