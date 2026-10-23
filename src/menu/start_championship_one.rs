//! `menu::start_championship_one` (0x4A25F4): the championship for one player (event mode 9): the
//! repairs first (with the entry's registers), then the common shape; then
//! `menu::run_championship`. The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. Refuses a missing player before anything, and what the
//! callees refuse when they are reached.
//! Hook removed 2026-10-22 (typed tables): the start menus run its `entry`
//! (`menu::table_calls`).

use crate::menu::start_event::{Starter, starter_hook};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_CHAMPIONSHIP_ONE: Starter = Starter {
    // pusha; call service::repair_car
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::One,
    },
};

starter_hook!("menu::start_championship_one", START_CHAMPIONSHIP_ONE, .private_copy(0xc2).relocate_calls(START_CHAMPIONSHIP_ONE.calls).manual_patch());
