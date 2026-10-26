//! `menu::start_championship_two` (0x4A27B5): the championship for two players in turn (event mode
//! 0xB); then `menu::run_championship`. The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. Refuses a missing player before anything, and what the
//! callees refuse when they are reached.

use crate::errors::Refused;
use crate::menu::screen_loop::Carried;
use crate::menu::start_event::{Starter, start_event};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_CHAMPIONSHIP_TWO: Starter = Starter {
    // pusha; mov dword_755954, 4
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::Two,
    },
};

/// The Rust version (`c` the carried values the item gets).
pub fn start_championship_two(c: &Carried) -> Result<(), Refused> {
    start_event(&START_CHAMPIONSHIP_TWO, c)
}
