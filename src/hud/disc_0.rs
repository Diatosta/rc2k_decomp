//! `hud::disc_0` (0x4B6BFD): the dial of layout 0 (`sub_4B1D8C`): its set-up (`hud::dials`), then `hud::draw_disc`.
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
        (From::Var(Slot::at(0x75C178)), Slot::at(0x75C050), false),
        (From::Var(Slot::at(0x75C17C)), Slot::at(0x75C04C), false),
        (From::Var(Slot::at(0x75C180)), Slot::at(0x75C044), false),
        (From::Var(Slot::at(0x75C184)), Slot::at(0x75C048), false),
    ],
};
