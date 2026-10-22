//! `menu::start_a8_split` (0x4A9D5C): the A8 championship for two players sharing the screen
//! (event mode 0xA); then `menu::run_a8_championship`. The shape is `menu::start_event`'s.
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

pub const START_A8_SPLIT: Starter = Starter {
    address: 0x4A9D5C,
    len: 0xE1,
    // pusha; mov dword_755954, 5
    entry: &[
        0x60, 0xC7, 0x05, 0x54, 0x59, 0x75, 0x00, 0x05, 0x00, 0x00, 0x00,
    ],
    calls: &[0x1F, 0x6B, 0x76, 0x84, 0xA8, 0xB3, 0xC1, 0xD0],
    kind: starters::Starter {
        championship: Championship::A8,
        seats: Seats::Split,
    },
};

starter_hook!("menu::start_a8_split", START_A8_SPLIT,);
