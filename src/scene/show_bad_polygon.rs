use crate::errors::BadInput;

/// The polygon in the vertex buffer dumped to the log (`scene::log_bad_polygon`).
pub fn show_bad_polygon() -> Result<(), BadInput> {
    crate::scene::log_bad_polygon::log_bad_polygon(crate::pointers::GamePtr::to(
        &crate::d3d::vertices_store::VERTICES.vertices,
    ))
}
