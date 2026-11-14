//! `d3d::draw_fan_unclipped`: draws the polygon in the vertex buffer: a triangle fan of
//! transformed and lit vertices, not to be clipped, lit or measured. The polygon vertex buffer
//! is the drawing part's (`workers::PartScratch`), the number of vertices the polygon's vertex
//! count. Two callers, `scene::draw_fan` and `race::draw_view_backdrop`. The fan is only
//! noted and the rasteriser draws it (`platform_3d::draw_fan`); the result is 0.

use crate::d3d::Vertex;
use crate::d3d::vertices_store::PolygonVertices;

/// The polygon as it stands in the buffer (the store's 64 vertices). More vertices than the
/// buffer holds is a count gone wrong: a refusal.
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
            crate::d3d::wire_vertex(
                [v.x.get(), v.y.get(), v.z.get(), v.rhw.get()],
                v.colour.get(),
                v.specular.get(),
                [v.u.get(), v.v.get()],
            )
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
