//! `hud::needle_1a` (0x4B6E67): the first needle of layout 1: its set-up (`hud::dials`), then `hud::draw_needle`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_needle` leaves them

use crate::hud::dials::{From, Setup};

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x10), |h| &h.needle_value, false),
        (
            From::Var(|h| &h.needle_1a[0]),
            |h| &h.needle_centre[0],
            false,
        ),
        (
            From::Var(|h| &h.needle_1a[1]),
            |h| &h.needle_centre[1],
            false,
        ),
        (From::Var(|h| &h.needle_1a[2]), |h| &h.needle_size, false),
    ],
};
