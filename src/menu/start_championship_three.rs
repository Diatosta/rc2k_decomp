//! `menu::start_championship_three` (0x4A28B4): the championship for three players (event mode
//! 0xC); then `menu::run_championship`. The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. The first five bytes hold no jump target or call. Ungated
//! (a whole event). Refuses a missing player before anything, and what the callees refuse when
//! they are reached. Shadow mode runs the original alone. Self-test: `start_event::test_starter`.

use crate::menu::start_event::{Starter, starter_hook};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_CHAMPIONSHIP_THREE: Starter = Starter {
    address: 0x4A28B4,
    len: 0x140,
    // pusha; mov dword_755954, 4
    entry: &[
        0x60, 0xC7, 0x05, 0x54, 0x59, 0x75, 0x00, 0x04, 0x00, 0x00, 0x00,
    ],
    calls: &[
        0x1F, 0x6B, 0x76, 0x88, 0xAC, 0xB7, 0xC5, 0xE9, 0xF4, 0x102, 0x111,
    ],
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::Three,
    },
};

starter_hook!("menu::start_championship_three", START_CHAMPIONSHIP_THREE,);
