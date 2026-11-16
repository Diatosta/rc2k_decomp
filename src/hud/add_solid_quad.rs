//! The quad record's texture places all on one texel (0xF8 << 16,
//! 8 << 16: a solid colour), then `hud::add_quad`. The caller sets the record's corners and
//! colour.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner_frame::Frame;

pub fn add_solid_quad(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    for (n, v) in rc2k_core::hud::quads::SOLID_PLACES {
        hud.block.quad.word(n).set(v);
    }
    crate::hud::add_quad::add_quad(frame, hud, pen)
}
