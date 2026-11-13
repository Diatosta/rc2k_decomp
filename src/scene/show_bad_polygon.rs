use crate::errors::BadInput;

/// The polygon of `view` in the vertex buffer dumped to the log (`scene::log_bad_polygon`).
pub fn show_bad_polygon(
    view: &crate::race::view_scratch::ViewScratch,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    screen: &crate::session::Screen,
) -> Result<(), BadInput> {
    crate::scene::log_bad_polygon::log_bad_polygon(view, screen, &buffer.vertices)
}
