//! `menu::start_championship_four` (0x4A29F4): the championship for four players (event mode 0xD);
//! then `menu::run_championship`. The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. Refuses a missing player before anything, and what the
//! callees refuse when they are reached.
//! Hook removed 2026-10-22 (typed tables): the start menus run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_loop::Carried;
use crate::menu::start_event::{Starter, start_event};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_CHAMPIONSHIP_FOUR: Starter = Starter {
    // pusha; mov dword_755954, 4
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::Four,
    },
};

/// The Rust version (`c` the carried values the item gets).
pub unsafe fn start_championship_four(c: &Carried) -> Result<(), Refused> {
    unsafe { start_event(&START_CHAMPIONSHIP_FOUR, c) }
}
