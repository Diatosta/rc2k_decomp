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

use crate::menu::start_event::{Starter, starter_hook};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_CHAMPIONSHIP_FOUR: Starter = Starter {
    address: 0x4A29F4,
    len: 0x181,
    // pusha; mov dword_755954, 4
    entry: &[
        0x60, 0xC7, 0x05, 0x54, 0x59, 0x75, 0x00, 0x04, 0x00, 0x00, 0x00,
    ],
    calls: &[
        0x1F, 0x6B, 0x76, 0x88, 0xAC, 0xB7, 0xC9, 0xED, 0xF8, 0x106, 0x12A, 0x135, 0x143, 0x152,
    ],
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::Four,
    },
};

starter_hook!("menu::start_championship_four", START_CHAMPIONSHIP_FOUR,);
