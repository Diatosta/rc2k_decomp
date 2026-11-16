//! `d3d::draw_fan`: draws the polygon in the vertex buffer, clipped by the device: a triangle fan
//! of transformed and lit vertices, with no flags. The polygon vertex buffer is the drawing part's
//! (`scene::workers::PartScratch`), the number of vertices the polygon's vertex count. The fan is
//! only noted (`d3d::recorder`) and the rasteriser draws it (`platform::direct3d::draw_fan`, held
//! to the screen); the result is 0.

use crate::d3d::draw_fan_unclipped::polygon;

/// Draws the polygon's fan.
pub fn draw_fan(
    part: &crate::scene::workers::PartScratch,
    poly: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
) {
    crate::platform::direct3d::draw_fan(part, &polygon(poly, buffer));
}
