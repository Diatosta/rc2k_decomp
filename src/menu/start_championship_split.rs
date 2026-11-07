//! `menu::start_championship_split` (0x4A26B6): the championship for two players sharing the
//! screen (event mode 0xA); then `menu::run_championship`. The shape is `menu::start_event`'s.
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

pub const START_CHAMPIONSHIP_SPLIT: Starter = Starter {
    // pusha; mov dword_755954, 4
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::Split,
    },
};

/// The Rust version.
pub fn start_championship_split(menus: &Menus) -> Result<(), Refused> {
    start_event(menus, &START_CHAMPIONSHIP_SPLIT)
}
