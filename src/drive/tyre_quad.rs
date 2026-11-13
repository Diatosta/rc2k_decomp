//! A screen quad of the tyre-change screen: the screen rectangle (x,
//! y, w, h, whole numbers) as the four vertices' places, z 0, rhw 1, the given
//! colour, the texture rectangle (`drive::tyre_parts`' `quad_tail`, scaled to the screen), drawn
//! (place 5, the device's DrawPrimitive: a call into the game).

use crate::d3d::vertices_store::PolygonVertices;
use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::race::viewer_screen_store::ViewerPanelBlock;
use crate::session::PlainDword;
use rc2k_core::drive::tyre_screen::quad_corners;

/// The texture rectangle at `uv`, the screen rectangle `rect`, the colour `colour`; For the
/// viewer's items, whose picture is a kept address (a rectangle or a switch table's entry).
pub fn tyre_quad(
    uv: [u32; 4],
    rect: &[PlainDword; 4],
    colour: u32,
    panel: &ViewerPanelBlock,
    system: &crate::system::owner::System,
    buffer: &PolygonVertices,
) -> Result<(), Refused> {
    quad_of(
        uv,
        rect.each_ref().map(PlainDword::get),
        colour,
        panel,
        system,
        buffer,
    )
}

/// `tyre_quad` on the rectangles' values.
pub fn quad_of(
    uv: [u32; 4],
    rect: [u32; 4],
    colour: u32,
    panel: &ViewerPanelBlock,
    system: &crate::system::owner::System,
    buffer: &PolygonVertices,
) -> Result<(), Refused> {
    let corners = quad_corners(rect);
    for (k, [x, y]) in corners.into_iter().enumerate() {
        buffer.vertices[k].x.set(x);
        buffer.vertices[k].y.set(y);
    }
    quad_words(buffer, colour);
    quad_tail(buffer, uv, panel, system);
    draw(buffer)
}
