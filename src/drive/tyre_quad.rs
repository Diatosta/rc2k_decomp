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
use crate::session::{Dword, read_all};
use rc2k_core::drive::tyre_screen::quad_corners;

/// The Rust version: the texture rectangle at `uv` (eax), the screen rectangle `rect`
/// (ebx), the colour `colour` (ecx); every register kept. For the viewer's items, whose
/// picture is a kept address (a rectangle or a switch table's entry).
pub fn tyre_quad(uv: [u32; 4], rect: &[Dword; 4], colour: u32) -> Result<(), Refused> {
    let rect = read_all(rect);
    quad_of(uv, rect, colour)
}

/// `tyre_quad` on rectangles held as cells.
// Converted: `rc2k_core::drive::tyre_screen::quad_corners` (the shared tail stays x87).
pub fn tyre_quad_cells(uv: &[Dword; 4], rect: &[Dword; 4], colour: u32) -> Result<(), Refused> {
    quad_of(read_all(uv), read_all(rect), colour)
}

fn quad_of(uv: [u32; 4], rect: [u32; 4], colour: u32) -> Result<(), Refused> {
    let corners = quad_corners(rect);
    for (k, [x, y]) in corners.into_iter().enumerate() {
        VERTICES.vertices[k].x.set_bits(x.to_bits());
        VERTICES.vertices[k].y.set_bits(y.to_bits());
    }
    quad_words(colour);
    quad_tail(uv);
    draw()
}
