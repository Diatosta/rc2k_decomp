//! `menu::start_a8_one` (0x4A9C9B): the A8 championship for one player (event mode 9): no
//! `dword_75649C`; unless a saved game goes on (`dword_755970` 1) `menu::begin_event`,
//! `reset_drivers`, `reset_driver_condition`; `dword_755970` 0; `menu::reset_cars_750724` before
//! the championship and `menu::show_in_game_options` after it; then `menu::run_a8_championship`.
//! The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. The first five bytes hold no jump target or call. Ungated
//! (a whole event). Refuses a missing player before anything, and what the callees refuse when
//! they are reached. Shadow mode runs the original alone. Self-test: `start_event::test_starter`.

use crate::menu::start_event::{Starter, starter_hook};
use rc2k_core::menu::starters::{self, Championship, Seats};

pub const START_A8_ONE: Starter = Starter {
    address: 0x4A9C9B,
    len: 0xC1,
    // pusha; mov dword_755954, 5
    entry: &[
        0x60, 0xC7, 0x05, 0x54, 0x59, 0x75, 0x00, 0x05, 0x00, 0x00, 0x00,
    ],
    calls: &[0x28, 0x2D, 0x32, 0x7E, 0x89, 0x97, 0xA6, 0xAB, 0xB0],
    kind: starters::Starter {
        championship: Championship::A8,
        seats: Seats::One,
    },
};

starter_hook!("menu::start_a8_one", START_A8_ONE,);
