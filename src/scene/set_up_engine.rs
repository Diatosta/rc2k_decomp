//! `scene::set_up_engine`: the 3D code's tables, and the cars' figures:
//! `scene::fill_engine_tables`, then `physics::acceleration_figures` (the cars' physics run for the figures
//! of the car specification screen, by the look of it). 22 call sites, among them
//! `menu::main_menu_loop` (twice) and `menu::play_films`.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::scene::fill_engine_tables::fill_engine_tables;
use crate::scene::owner::Scene;

/// The tables, then the cars' figures (`physics::acceleration_figures`).
pub fn set_up_engine_in(race: &Race, scene: &Scene) -> Result<(), Refused> {
    fill_engine_tables(race, scene);
    crate::physics::acceleration_figures::acceleration_figures(
        race.physics.car_specs(),
        &race.course,
        &race.shared,
        race.gear(),
        scene,
    )
}
