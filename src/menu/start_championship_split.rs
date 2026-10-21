//! `menu::start_championship_split` (0x4A26B6): the championship for two players sharing the
//! screen (event mode 0xA); then `menu::run_championship`. The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. The first five bytes hold no jump target or call. Ungated
//! (a whole event). Refuses a missing player before anything, and what the callees refuse when
//! they are reached. Shadow mode runs the original alone. Self-test: `start_event::test_starter`.

use crate::menu::start_event::{Starter, starter_hook};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_CHAMPIONSHIP_SPLIT: Starter = Starter {
    address: 0x4A26B6,
    len: 0xFF,
    // pusha; mov dword_755954, 4
    entry: &[
        0x60, 0xC7, 0x05, 0x54, 0x59, 0x75, 0x00, 0x04, 0x00, 0x00, 0x00,
    ],
    calls: &[0x1F, 0x6B, 0x76, 0x84, 0xA8, 0xB3, 0xC1, 0xD0],
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::Split,
    },
};

starter_hook!("menu::start_championship_split", START_CHAMPIONSHIP_SPLIT,);
