//! `hud::add_triangle` (0x4B6DA7): `g_polygonVertexCount` 3 and `hud::add_polygon`.
//!
//! In:  nothing
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4B6D10` (0x4B6D03).

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::race::owner_frame::Frame;
use crate::scene::draw_store::DRAW;

/// The Rust version (see the header).
pub fn add_triangle(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    DRAW.polygon_vertex_count.set_bits(3u32);
    crate::hud::add_polygon::add_polygon(frame, hud)
}
