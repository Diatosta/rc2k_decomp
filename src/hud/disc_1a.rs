//! `hud::disc_1a` (0x4B6B5B): the first dial of layout 1 (`sub_4B2743`): its set-up (`hud::dials`), then `hud::draw_disc`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_disc` leaves them

use crate::hud::dials::{From, Setup};
use crate::hud::store::Slot;

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x34), Slot::at(0x75C054), false),
        (From::Field(0x38), Slot::at(0x75C058), false),
        (From::Field(0x3C), Slot::at(0x75C05C), false),
        (From::Var(Slot::at(0x75C158)), Slot::at(0x75C050), false),
        (From::Var(Slot::at(0x75C15C)), Slot::at(0x75C04C), false),
        (From::Var(Slot::at(0x75C160)), Slot::at(0x75C044), false),
        (From::Var(Slot::at(0x75C164)), Slot::at(0x75C048), false),
    ],
};
