//! The HUD's quad record (`HudBlock.quad.alpha`: alpha, left, top, right, bottom, colour, then
//! the four corners' texture places as whole numbers) made the four polygon vertices (the places
//! over `HudBlock.uv_scale_x` and `uv_scale_y`; the colour's low 24 bits under the alpha) and
//! `hud::add_polygon`.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner_frame::Frame;

use rc2k_core::hud::quads;

pub fn add_quad(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    let q: [u32; quads::RECORD] = hud.block.quad.words();
    let (a, b) = (hud.block.uv_scale_x.get(), hud.block.uv_scale_y.get());
    let (vertices, _, _) = quads::quad_vertices(&q, a, b);
    for (n, v) in vertices.into_iter().enumerate() {
        let at = &pen.vertices.vertices[n];
        at.x.set(v.x);
        at.y.set(v.y);
        at.colour.set(v.colour);
        at.u.set(v.u);
        at.v.set(v.v);
    }
    pen.polygons.polygon_vertex_count.set(4u32);
    crate::hud::add_polygon::add_polygon(frame, hud, pen)
}
