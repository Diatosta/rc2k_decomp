//! `menu::start_a8_one` (0x4A9C9B): the A8 championship for one player (event mode 9): no
//! `dword_75649C`; unless a saved game goes on (`dword_755970` 1) `menu::begin_event`,
//! `reset_drivers`, `reset_driver_condition`; `dword_755970` 0; `menu::reset_cars_750724` before
//! the championship and `menu::show_in_game_options` after it; then `menu::run_a8_championship`.
//! The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. Refuses a missing player before anything, and what the
//! callees refuse when they are reached.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::start_event::{Starter, start_event};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_A8_ONE: Starter = Starter {
    // pusha; mov dword_755954, 5
    kind: starters::Starter {
        championship: Championship::A8,
        seats: Seats::One,
    },
};

/// The Rust version.
pub fn start_a8_one(menus: &Menus) -> Result<(), Refused> {
    start_event(menus, &START_A8_ONE)
}
