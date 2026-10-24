//! `menu::start_a8_two` (0x4A9E3D): the A8 championship for two players in turn (event mode 0xB);
//! then `menu::run_a8_championship`. The shape is `menu::start_event`'s.
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

pub const START_A8_TWO: Starter = Starter {
    // pusha; mov dword_755954, 5
    kind: starters::Starter {
        championship: Championship::A8,
        seats: Seats::Two,
    },
};

/// The Rust version (`c` the carried values the item gets).
pub unsafe fn start_a8_two(c: &Carried) -> Result<(), Refused> {
    unsafe { start_event(&START_A8_TWO, c) }
}
