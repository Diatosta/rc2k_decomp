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

use crate::framework::{Regs, rva, standalone};

const VERTICES_RVA: usize = rva(0x764F2C); // off_764F2C: g_hudVertices
const INDICES_RVA: usize = rva(0x764F30); // off_764F30: g_hudIndices
const VERTEX_COUNT_RVA: usize = rva(0x764F24); // g_hudVertexCount
const INDEX_COUNT_RVA: usize = rva(0x764F28); // g_hudIndexCount

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if standalone() {
        unsafe {
            use crate::framework::game_read;
            // The two pointers are the game's: in the standalone's image they were moved
            // with it.
            crate::platform_3d::draw_indexed_list(
                game_read(VERTICES_RVA),
                game_read(VERTEX_COUNT_RVA),
                game_read(INDICES_RVA),
                game_read(INDEX_COUNT_RVA),
            )
        };
        r.eax = 0;
        return Ok(());
    }
    unsafe { crate::framework::call_game(r, crate::framework::game_addr(rva(0x45A869))) };
    Ok(())
}
