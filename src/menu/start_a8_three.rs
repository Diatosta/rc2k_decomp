//! `menu::start_a8_three` (0x4A9F1E): the A8 championship for three players (event mode 0xC); then
//! `menu::run_a8_championship`. The shape is `menu::start_event`'s.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through a start menu's table. The first five bytes hold no jump target or call. Ungated
//! (a whole event). Refuses a missing player before anything, and what the callees refuse when
//! they are reached. Shadow mode runs the original alone. Self-test: `start_event::test_starter`.

use crate::menu::start_event::{Starter, starter_hook};

pub const START_A8_THREE: Starter = Starter {
    address: 0x4A9F1E,
    len: 0x122,
    // pusha; mov dword_755954, 5
    entry: &[
        0x60, 0xC7, 0x05, 0x54, 0x59, 0x75, 0x00, 0x05, 0x00, 0x00, 0x00,
    ],
    calls: &[
        0x1F, 0x6B, 0x76, 0x88, 0xAC, 0xB7, 0xC5, 0xE9, 0xF4, 0x102, 0x111,
    ],
    event_type: 5,
    players: 3,
    mode: 0xC,
    first: false,
};

starter_hook!("menu::start_a8_three", START_A8_THREE,);
