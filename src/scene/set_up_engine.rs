//! `scene::set_up_engine` (0x442AF6): the 3D code's tables, and the cars' figures.
//!
//! `scene::fill_engine_tables`, then `physics::test_drive` (the cars' physics run for the figures
//! of the car specification screen, by the look of it; not ported). 22 call sites, among them
//! `menu::main_menu_loop` (twice) and `menu::play_films`. The registers above are the
//! original's, for reference; a call by address goes through `routes.rs`.

use crate::errors::Refused;
use crate::race::owner::race;
use crate::scene::fill_engine_tables::fill_engine_tables;

/// The Rust version: the tables, then the cars' figures (`physics::test_drive`). The
/// registers the original leaves nothing reads (see `physics::test_drive`).
pub fn set_up_engine() -> Result<(), Refused> {
    let scene = crate::scene::owner::scene();
    let race = race();
    fill_engine_tables(race);
    crate::physics::test_drive::test_drive(race, scene)
}
