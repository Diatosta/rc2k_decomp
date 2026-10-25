//! `hud::disc_0` (0x4B6BFD): the dial of layout 0 (`sub_4B1D8C`): its set-up (`hud::dials`), then `hud::draw_disc`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_disc` leaves them

use crate::hud::dials::{From, Setup};

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x34), |h| &h.disc_texture_shift[0], false),
        (From::Field(0x38), |h| &h.disc_texture_shift[1], false),
        (From::Field(0x3C), |h| &h.disc_turn, false),
        (From::Var(|h| &h.disc_0[0]), |h| &h.disc_size, false),
        (
            From::Var(|h| &h.disc_0[1]),
            |h| &h.disc_texture_scale,
            false,
        ),
        (From::Var(|h| &h.disc_0[2]), |h| &h.disc_centre[0], false),
        (From::Var(|h| &h.disc_0[3]), |h| &h.disc_centre[1], false),
    ],
};
