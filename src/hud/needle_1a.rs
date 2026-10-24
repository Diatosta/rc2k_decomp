//! `hud::needle_1a` (0x4B6E67): the first needle of layout 1: its set-up (`hud::dials`), then `hud::draw_needle`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_needle` leaves them
//!
//! The shape through `part` (1), in the game its Rust version. Shadow mode runs the
//! original alone (it draws). Self-test: `hud::dials::test`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::hud::dials::{From, Setup};
use crate::hud::store::Slot;

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x10), Slot::at(0x75C2E0), false),
        (From::Var(Slot::at(0x75C30C)), Slot::at(0x75C2A0), false),
        (From::Var(Slot::at(0x75C310)), Slot::at(0x75C2A4), false),
        (From::Var(Slot::at(0x75C314)), Slot::at(0x75C2B8), false),
    ],
};
