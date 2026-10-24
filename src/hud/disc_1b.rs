//! `hud::disc_1b` (0x4B6BAC): the second dial of layout 1: its set-up (`hud::dials`), then `hud::draw_disc`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_disc` leaves them

use crate::hud::dials::{From, Setup};
use crate::hud::store::Slot;

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x5C), Slot::at(0x75C054), false),
        (From::Field(0x60), Slot::at(0x75C058), false),
        (From::Field(0x64), Slot::at(0x75C05C), false),
        (From::Var(Slot::at(0x75C168)), Slot::at(0x75C050), false),
        (From::Var(Slot::at(0x75C16C)), Slot::at(0x75C04C), false),
        (From::Var(Slot::at(0x75C170)), Slot::at(0x75C044), false),
        (From::Var(Slot::at(0x75C174)), Slot::at(0x75C048), false),
    ],
};
