//! `hud::needle_2a` (0x4B6DEC): the first needle of layout 2: its set-up (`hud::dials`), then `hud::draw_needle`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_needle` leaves them

use crate::hud::dials::{From, Setup};
use crate::hud::store::Slot;

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x10), Slot::at(0x75C2E0), false),
        (From::Var(Slot::at(0x75C2F0)), Slot::at(0x75C2A0), false),
        (From::Var(Slot::at(0x75C2F4)), Slot::at(0x75C2A4), false),
        (From::Var(Slot::at(0x75C2F8)), Slot::at(0x75C2A4), true),
        (From::Var(Slot::at(0x75C2FC)), Slot::at(0x75C2B8), false),
    ],
};
