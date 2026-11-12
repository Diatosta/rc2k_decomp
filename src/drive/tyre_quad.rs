//! `drive::tyre_quad` (0x41DD3B): a screen quad of the tyre-change screen: the screen rectangle (x,
//! y, w, h, whole numbers) as the four vertices' places (`g_polygonVertices`), z 0, rhw 1, the given
//! colour, the texture rectangle (`drive::tyre_parts`' `quad_tail`, scaled to the screen), drawn
//! (place 5, the device's DrawPrimitive: a call into the game).
//!
//! Takes: the texture rectangle, the screen rectangle, the colour
//!
//! Callers: `drive::tyre_screen` (two), `drive::tyre_screen_parts` (ten).

use crate::d3d::vertices_store::VERTICES;
use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::session::{Dword, read_all};
use rc2k_core::drive::tyre_screen::quad_corners;

/// The texture rectangle at `uv`, the screen rectangle `rect`, the colour `colour`; For the
/// viewer's items, whose picture is a kept address (a rectangle or a switch table's entry).
pub fn tyre_quad(
    uv: [u32; 4],
    rect: &[Dword; 4],
    colour: u32,
    panel: &ViewerPanelBlock,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let rect = read_all(rect);
    quad_of(uv, rect, colour, panel, system)
}

/// `tyre_quad` on rectangles held as cells.
// Converted: `rc2k_core::drive::tyre_screen::quad_corners` (the shared tail stays x87).
pub fn tyre_quad_cells(
    uv: &[Dword; 4],
    rect: &[Dword; 4],
    colour: u32,
    panel: &ViewerPanelBlock,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    quad_of(read_all(uv), read_all(rect), colour, panel, system)
}

fn quad_of(
    uv: [u32; 4],
    rect: [u32; 4],
    colour: u32,
    panel: &ViewerPanelBlock,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let corners = quad_corners(rect);
    for (k, [x, y]) in corners.into_iter().enumerate() {
        VERTICES.vertices[k].x.set(x);
        VERTICES.vertices[k].y.set(y);
    }
    quad_words(colour);
    quad_tail(uv, panel, system);
    draw()
}
