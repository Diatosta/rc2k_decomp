//! `d3d::draw_hud_batch` (0x45A869): draws the HUD's batch, an indexed list of triangles.
//!
//! In:  nothing: the vertices at `off_764F2C` (`g_hudVertices`), their number in
//!      `g_hudVertexCount` (0x764F24), the indices (16 bits) at `off_764F30`
//!      (`g_hudIndices`), their number in `g_hudIndexCount` (0x764F28)
//! Out: eax = Direct3D's result (0: drawn); ecx, edx as Direct3D leaves them
//! The device's DrawIndexedPrimitive with a triangle list (4) of transformed and lit
//! vertices (3) and the flags 0x1C. One caller, `hud::flush_batch`.
//! Platform: the original always runs. In the standalone program the rasteriser draws
//! each triangle (`platform_3d::draw_indexed_list`) and the result is 0.

use crate::hud::store::{HUD, indices, vertices};

/// Draws the HUD's batch. Its leftovers (eax 0, ecx, edx) are read by none of the
/// original's callers (register liveness, 2026-10-23).
pub fn draw_hud_batch() {
    // The vertices and indices are the HUD block's (`hud::store`); the image's two
    // pointers to them (`off_764F2C`, `off_764F30`) are no longer read.
    crate::platform_3d::draw_indexed_list(
        vertices() as u32,
        HUD.vertex_count.get(),
        indices() as u32,
        HUD.index_count.get(),
    );
}
