//! The quad record's texture places all on one texel (0xF8 << 16,
//! 8 << 16: a solid colour), then `hud::add_quad`. The caller sets the record's corners and
//! colour.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;

pub fn add_solid_quad(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    let texel = rc2k_core::hud::quads::SOLID_TEXEL;
    hud.block.quad.set_places([texel; 4]);
    crate::hud::add_quad::add_quad(frame, hud, pen)
}
