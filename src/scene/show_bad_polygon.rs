use crate::errors::BadInput;

/// The polygon in the vertex buffer dumped to the log (`scene::log_bad_polygon`).
pub fn show_bad_polygon(
    poly: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    screen: &crate::session::Screen,
) -> Result<(), BadInput> {
    crate::scene::log_bad_polygon::log_bad_polygon(poly, screen, &buffer.vertices)
}
