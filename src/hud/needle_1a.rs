//! `hud::needle_1a` (0x4B6E67): the first needle of layout 1: its set-up (`hud::dials`), then `hud::draw_needle`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_needle` leaves them

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
