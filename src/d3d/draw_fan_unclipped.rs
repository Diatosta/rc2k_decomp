//! `d3d::draw_fan_unclipped`: draws the polygon in the vertex buffer: a triangle fan of transformed
//! and lit vertices, not to be clipped, lit or measured. The polygon vertex buffer is the drawing
//! part's (`scene::scratch_store::PartScratch`), the number of vertices the polygon's vertex count. The
//! fan is only noted and the rasteriser draws it (`d3d::device::draw_fan`); the result is 0.

use crate::d3d::Vertex;
use crate::d3d::vertices_store::PolygonVertices;

/// Vertices a polygon holds without the heap: what clipping against the depth and screen edges
/// makes of a face stays under it.
const INLINE_VERTICES: usize = 12;

/// The polygon as it stands in the buffer (the store's 64 vertices). More vertices than the
/// buffer holds is a count gone wrong: a refusal.
pub fn polygon(
    poly: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &PolygonVertices,
) -> smallvec::SmallVec<[Vertex; INLINE_VERTICES]> {
    let count = poly.scalars.borrow().polygon_vertex_count as usize;
    let vertices = &buffer.vertices;
    if count > vertices.len() {
        crate::program_log::log(&format!(
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
            crate::d3d::wire_vertex(
                [v.x.get(), v.y.get(), v.z.get(), v.rhw.get()],
                v.colour.get(),
                v.specular.get(),
                [v.u.get(), v.v.get()],
            )
        })
        .collect()
}

/// Draws the polygon's fan.
pub fn draw_fan_unclipped(
    part: &crate::scene::scratch_store::PartScratch,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &PolygonVertices,
) {
    crate::d3d::device::draw_fan(part, &polygon(poly, buffer));
}
