use crate::errors::BadInput;

/// The polygon in the vertex buffer dumped to the log (`scene::log_bad_polygon`).
pub fn show_bad_polygon(poly: &crate::scene::polygon_work::PolygonWork) -> Result<(), BadInput> {
    crate::scene::log_bad_polygon::log_bad_polygon(
        poly,
        &crate::d3d::vertices_store::VERTICES.vertices,
    )
}
