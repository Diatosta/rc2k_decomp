//! `menu::start_championship_two` (0x4A27B5): the championship for two players in turn (event mode
//! 0xB); then `menu::run_championship`. The shape is `menu::start_event`'s.
//!
//! Reached through a start menu's table. Refuses a missing player before anything, and what the
//! callees refuse when they are reached.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::start_event::{Starter, start_event};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_CHAMPIONSHIP_TWO: Starter = Starter {
    // pusha; mov dword_755954, 4
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::Two,
    },
};

/// The Rust version.
pub fn start_championship_two(menus: &Menus) -> Result<(), Refused> {
    start_event(menus, &START_CHAMPIONSHIP_TWO)
}
