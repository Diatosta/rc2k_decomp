//! `hud::disc_2b` (0x4B6B0A): the second dial of layout 2: its set-up (`hud::dials`), then `hud::draw_disc`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_disc` leaves them

use crate::hud::dials::{From, Setup};

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x5C), |h| &h.disc_texture_shift[0], false),
        (From::Field(0x60), |h| &h.disc_texture_shift[1], false),
        (From::Field(0x64), |h| &h.disc_turn, false),
        (From::Var(|h| &h.disc_2b[0]), |h| &h.disc_size, false),
        (
            From::Var(|h| &h.disc_2b[1]),
            |h| &h.disc_texture_scale,
            false,
        ),
        (From::Var(|h| &h.disc_2b[2]), |h| &h.disc_centre[0], false),
        (From::Var(|h| &h.disc_2b[3]), |h| &h.disc_centre[1], false),
    ],
};
