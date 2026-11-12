//! `d3d::draw_fan_unclipped` (0x41AD9C): draws the polygon in the vertex buffer.
//!
//! In:  the polygon vertex buffer (the drawing part's, `workers::PartScratch`), the number of
//!      vertices in the polygon's vertex count
//! The device's DrawPrimitive with a triangle fan (6) of transformed and lit vertices
//! (3) and the flags 0x1C (not to be clipped, lit or measured). Two callers,
//! `scene::draw_polygon_fan` and `race::draw_view_backdrop`.
//! Platform: the original always runs, here the fan is only noted. In the standalone
//! program the rasteriser draws it (`platform_3d::draw_fan`) and the result is 0.

use crate::d3d::Vertex;
use crate::d3d::vertices_store::PolygonVertices;

/// The polygon as it stands in the buffer (the store's `TLVertex[64]`). More vertices than
/// the buffer holds is a count gone wrong (the original reads on past it, into other
/// variables): a refusal.
pub fn polygon(
    poly: &crate::scene::polygon_work::PolygonWork,
    buffer: &PolygonVertices,
) -> smallvec::SmallVec<[Vertex; 6]> {
    let count = poly.polygon_vertex_count.get() as usize;
    let vertices = &buffer.vertices;
    if count > vertices.len() {
        crate::framework::log(&format!(
            "d3d::draw_fan_unclipped: a polygon of {count} vertices, the buffer holds {}",
            vertices.len()
        ));
        crate::fatal::raise::end(
            "d3d::draw_fan_unclipped",
            crate::errors::BadInput("a polygon of more vertices than its buffer holds"),
        );
    }
    vertices[..count]
        .iter()
        .map(|v| {
            [
                v.x.bits(),
                v.y.bits(),
                v.z.bits(),
                v.rhw.bits(),
                v.colour.get(),
                v.specular.get(),
                v.u.bits(),
                v.v.bits(),
            ]
        })
        .collect()
}

/// The fan drawn: Direct3D's result, always 0 here.
pub fn draw_fan_unclipped(
    part: &crate::workers::PartScratch,
    poly: &crate::scene::polygon_work::PolygonWork,
    buffer: &PolygonVertices,
) -> u32 {
    crate::platform_3d::draw_fan(part, &polygon(poly, buffer));
    0
}
