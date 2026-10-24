//! `d3d::draw_hud_batch` (0x45A869): draws the HUD's batch, an indexed list of triangles.
//!
//! In:  nothing: the vertices at `off_764F2C` (`g_hudVertices`), their number in
//!      `g_hudVertexCount` (0x764F24), the indices (16 bits) at `off_764F30`
//!      (`g_hudIndices`), their number in `g_hudIndexCount` (0x764F28)
//! Out: eax = Direct3D's result (0: drawn); ecx, edx as Direct3D leaves them
//! The device's DrawIndexedPrimitive with a triangle list (4) of transformed and lit
//! vertices (3) and the flags 0x1C. One caller, `hud::flush_batch`.
//! Platform: the original always runs. In the standalone program the rasteriser draws
//! each triangle (`platform_3d::draw_indexed_list`) and the result is 0. No self-test.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::hud::store::{Slot, game_read, indices, vertices};

const VERTEX_COUNT: Slot = Slot::at(0x764F24); // g_hudVertexCount
const INDEX_COUNT: Slot = Slot::at(0x764F28); // g_hudIndexCount

/// Draws the HUD's batch. Its leftovers (eax 0, ecx, edx) are read by none of the
/// original's callers (register liveness, 2026-10-23).
pub fn draw_hud_batch() {
    unsafe {
        // The vertices and indices are the HUD block's (`hud::store`); the image's two
        // pointers to them (`off_764F2C`, `off_764F30`) are no longer read.
        crate::platform_3d::draw_indexed_list(
            vertices() as u32,
            game_read(VERTEX_COUNT),
            indices() as u32,
            game_read(INDEX_COUNT),
        )
    };
}
