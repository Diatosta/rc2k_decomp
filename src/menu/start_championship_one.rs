//! `menu::start_championship_one` (0x4A25F4): the championship for one player (event mode 9): the
//! repairs first (with the entry's registers), then the common shape; then
//! `menu::run_championship`. The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. A call at +1: a private copy (its calls relocated) and a
//! jump written by hand. Ungated (a whole event). Refuses a missing player before anything, and
//! what the callees refuse when they are reached. Shadow mode runs the original alone. Self-test:
//! `start_event::test_starter`.

use crate::menu::start_event::{Starter, starter_hook};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_CHAMPIONSHIP_ONE: Starter = Starter {
    address: 0x4A25F4,
    len: 0xC2,
    // pusha; call service::repair_car
    entry: &[0x60, 0xE8, 0x69, 0x49, 0xFE, 0xFF],
    calls: &[0x1, 0x6B, 0x76, 0x84, 0x93],
    kind: starters::Starter {
        championship: Championship::First,
        seats: Seats::One,
    },
};

starter_hook!("menu::start_championship_one", START_CHAMPIONSHIP_ONE, .private_copy(0xc2).relocate_calls(START_CHAMPIONSHIP_ONE.calls).manual_patch());
