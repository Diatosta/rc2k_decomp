//! `hud::needle_2b` (0x4B6E34): the second needle of layout 2: its set-up (`hud::dials`), then `hud::draw_needle`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_needle` leaves them

use crate::hud::dials::{From, Setup};
use crate::hud::store::Slot;

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x40), Slot::at(0x75C2E0), false),
        (From::Var(Slot::at(0x75C300)), Slot::at(0x75C2A0), false),
        (From::Var(Slot::at(0x75C304)), Slot::at(0x75C2A4), false),
        (From::Var(Slot::at(0x75C308)), Slot::at(0x75C2B8), false),
    ],
};
