//! `menu::start_a8_four` (0x4AA040): the A8 championship for four players (event mode 0xD); then
//! `menu::run_a8_championship`. The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. The first five bytes hold no jump target or call. Ungated
//! (a whole event). Refuses a missing player before anything, and what the callees refuse when
//! they are reached. Shadow mode runs the original alone. Self-test: `start_event::test_starter`.

use crate::menu::start_event::{Starter, starter_hook};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_A8_FOUR: Starter = Starter {
    address: 0x4AA040,
    len: 0x163,
    // pusha; mov dword_755954, 5
    entry: &[
        0x60, 0xC7, 0x05, 0x54, 0x59, 0x75, 0x00, 0x05, 0x00, 0x00, 0x00,
    ],
    calls: &[
        0x1F, 0x6B, 0x76, 0x88, 0xAC, 0xB7, 0xC9, 0xED, 0xF8, 0x106, 0x12A, 0x135, 0x143, 0x152,
    ],
    kind: starters::Starter {
        championship: Championship::A8,
        seats: Seats::Four,
    },
};

starter_hook!("menu::start_a8_four", START_A8_FOUR,);
