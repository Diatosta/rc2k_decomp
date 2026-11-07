use crate::errors::BadInput;

/// The polygon in the vertex buffer dumped to the log (`scene::log_bad_polygon`).
pub fn show_bad_polygon(scene: &crate::scene::owner::Scene) -> Result<(), BadInput> {
    crate::scene::log_bad_polygon::log_bad_polygon(
        &crate::d3d::vertices_store::VERTICES.vertices,
        scene,
    )
}
