//! `menu::start_championship_one` (0x4A25F4): the championship for one player (event mode 9): the
//! repairs first (with the entry's registers), then the common shape; then
//! `menu::run_championship`. The shape is `menu::start_event`'s.
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

pub const START_CHAMPIONSHIP_ONE: Starter = Starter {
    // pusha; call service::repair_car
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::One,
    },
};

/// The Rust version (`c` the carried values the item gets).
pub fn start_championship_one(c: &Carried) -> Result<(), Refused> {
    start_event(&START_CHAMPIONSHIP_ONE, c)
}
