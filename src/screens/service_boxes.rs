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

const QUAD_IN: usize = 0x623DE8; // dword_623DE8: colour, see-through; x at +0x28 / +0x2C, y at +0x38 / +0x3C

/// One quad: x .. x + w, y .. y + h, the colour and see-through, then `draw_tinted_quad`.
pub fn quad(x: u32, y: u32, w: u32, h: u32, colour: u32, see: u32) -> Result<(), Refused> {
    let put =
        |at: usize, v: u32| unsafe { crate::framework::game_write(crate::framework::rva(at), v) };
    put(QUAD_IN + 0x28, x);
    put(QUAD_IN + 0x38, y);
    put(QUAD_IN + 0x2C, x.wrapping_add(w));
    put(QUAD_IN + 0x3C, y.wrapping_add(h));
    put(QUAD_IN, colour);
    put(QUAD_IN + 4, see);
    unsafe { crate::scene::draw_tinted_quad::draw_tinted_quad() }
}
