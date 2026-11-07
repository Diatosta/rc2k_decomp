//! `menu::start_a8_four` (0x4AA040): the A8 championship for four players (event mode 0xD); then
//! `menu::run_a8_championship`. The shape is `menu::start_event`'s.
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

pub const START_A8_FOUR: Starter = Starter {
    // pusha; mov dword_755954, 5
    kind: starters::Starter {
        championship: Championship::A8,
        seats: Seats::Four,
    },
};

/// The Rust version.
pub fn start_a8_four(menus: &Menus) -> Result<(), Refused> {
    start_event(menus, &START_A8_FOUR)
}
