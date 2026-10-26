//! `drive::tyre_quad` (0x41DD3B): a screen quad of the tyre-change screen: the screen
//! rectangle at ebx (x, y, w, h, whole numbers) as the four vertices' places
//! (`g_polygonVertices`), z 0, rhw 1, the colour ecx, the texture rectangle at eax
//! (`drive::tyre_parts`' `quad_tail`, scaled to the screen), drawn (place 5, the device's
//! DrawPrimitive: a call into the game). All in `pusha` / `popa`.
//!
//! In:  eax the texture rectangle, ebx the screen rectangle, ecx the colour
//! Out: every register kept; no flags read
//!
//! Callers: `drive::tyre_screen` (two), `drive::tyre_screen_parts` (ten).

use crate::d3d::vertices_store::VERTICES;
use crate::drive::tyre_parts::*;
use crate::errors::Refused;

use crate::pointers::GamePtr;
use rc2k_core::drive::tyre_screen::quad_corners;

/// The Rust version: the texture rectangle at `uv` (eax), the screen rectangle at `rect`
/// (ebx), the colour `colour` (ecx); every register kept.
// Converted: `rc2k_core::drive::tyre_screen::quad_corners` (the shared tail stays x87).
pub fn tyre_quad(uv: u32, rect: u32, colour: u32) -> Result<(), Refused> {
    let rect: [u32; 4] = GamePtr::new(rect).read("a tyre's rectangle")?;
    let corners = quad_corners(rect);
    for (k, [x, y]) in corners.into_iter().enumerate() {
        VERTICES.vertices[k].x.set_bits(x.to_bits());
        VERTICES.vertices[k].y.set_bits(y.to_bits());
    }
    quad_words(colour);
    quad_tail(uv);
    draw()
}
