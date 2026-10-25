//! The service area's boxes (`screens::draw_service_boxes` and its four frames): flat see-through
//! quads (`scene::draw_tinted_quad`) laid out by long tables of the original's code.

use crate::errors::Refused;
use crate::session::Dword;

/// A variable of the game.
pub fn rd(a: &Dword) -> u32 {
    a.get()
}

pub fn set(a: &Dword, v: u32) {
    a.set(v);
}

/// The dword at `p` (the original's `[esi]`).
pub fn at_esi(p: u32) -> u32 {
    unsafe { (p as *const u32).read_unaligned() }
}

/// One quad: x .. x + w, y .. y + h, the colour and see-through, then `draw_tinted_quad`.
pub fn quad(x: u32, y: u32, w: u32, h: u32, colour: u32, see: u32) -> Result<(), Refused> {
    // dword_623DE8: colour, see-through; x at +0x28 / +0x2C, y at +0x38 / +0x3C (`quad_corners`)
    let q = &crate::race::vars_store::RACE_VARS;
    q.quad_corners[0].set(x);
    q.quad_corners[4].set(y);
    q.quad_corners[1].set(x.wrapping_add(w));
    q.quad_corners[5].set(y.wrapping_add(h));
    q.quad_colour[0].set(colour);
    q.quad_colour[1].set(see);
    unsafe { crate::scene::draw_tinted_quad::draw_tinted_quad() }
}
