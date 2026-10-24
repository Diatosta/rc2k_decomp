use crate::errors::BadInput;
use crate::framework::{game_addr, rva};

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices

/// The polygon in the vertex buffer dumped to the log (`scene::log_bad_polygon`).
pub fn show_bad_polygon() -> Result<(), BadInput> {
    unsafe { crate::scene::log_bad_polygon::log_bad_polygon(game_addr(VERTICES_RVA) as u32) }
}
