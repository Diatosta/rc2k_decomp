//! `hud::add_triangle` (0x4B6DA7): `g_polygonVertexCount` 3 and `hud::add_polygon`.
//!
//! Caller: `hud::disc_vertex` (0x4B6D03).

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::race::owner_frame::Frame;

pub fn add_triangle(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set(3u32);
    crate::hud::add_polygon::add_polygon(frame, hud)
}
