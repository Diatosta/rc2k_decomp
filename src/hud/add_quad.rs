//! `hud::add_quad` (0x4B72D1): the HUD's quad record (`dword_75BFB0`: alpha, left, top, right,
//! bottom, colour, then the four corners' texture places as whole numbers) made the four vertices
//! of `g_polygonVertices` (the places over `flt_75C00C` and over `flt_75C010`; the colour's low 24
//! bits under the alpha) and `hud::add_polygon`.
//!
//! Callers: `hud::add_scaled_sprite`, `hud::add_solid_quad`, `hud::add_sprite` (reached through
//! code at 0x4B7163, 0x4B720F, 0x4B7266).

use crate::d3d::vertices_store::VERTICES;
use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::race::owner_frame::Frame;

use rc2k_core::hud::quads;

// Converted: `rc2k_core::hud::quads::quad_vertices`
pub fn add_quad(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    let q: [u32; quads::RECORD] = hud.block.quad.words();
    let (a, b) = (hud.block.uv_scale_x.get(), hud.block.uv_scale_y.get());
    let (vertices, _, _) = quads::quad_vertices(&q, a, b);
    for (n, v) in vertices.into_iter().enumerate() {
        let at = &VERTICES.vertices[n];
        at.x.set_bits(v.x.to_bits());
        at.y.set_bits(v.y.to_bits());
        at.colour.set(v.colour);
        at.u.set_bits(v.u.to_bits());
        at.v.set_bits(v.v.to_bits());
    }
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set_bits(4u32);
    crate::hud::add_polygon::add_polygon(frame, hud)
}
