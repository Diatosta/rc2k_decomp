//! `physics::update_car_angles` (0x43BEDC): `physics::car_angles` when `dword_71A6C8` is
//! set or `dword_71A22C` is 0; else nothing.
//!
//! In:  nothing
//! Out: when run, esi = `g_crashMatrix`; the rest kept; no flags read (the callers' next
//!      instructions are `lea` and `mov`)
//!
//! Callers: `sub_43B635` (0x43B83B, 0x43B8B4, after `race::invert_car_matrix`).

use crate::physics::car_angles::car_angles;

use crate::race::owner::Race;
use rc2k_core::physics::motion::angles_wanted as wanted;

/// The Rust version.
// Converted: `rc2k_core::physics::motion::angles_wanted`.
pub fn update_car_angles(race: &Race) {
    let (force, hold) = (
        race.shared.car.camera_mode_is_2_or_3.get(),
        race.shared.car.all_wheels_off.get(),
    );
    if wanted(force, hold) {
        car_angles(race);
    }
}
