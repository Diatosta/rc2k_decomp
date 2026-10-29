//! `d3d::draw_marks_windscreen_drops` (0x41ADB9): draws the marks' windscreen drops, an indexed list of
//! triangles.
//!
//! In:  nothing: the vertices at `unk_625A44`, their number in `dword_625A40`, the indices
//!      (16 bits) at `g_marksWindscreenDropFans`, their number in `dword_62423C`
//! Out: eax = Direct3D's result (0: drawn); ecx, edx as Direct3D leaves them
//! The device's (`g_d3dDevice`, an `IDirect3DDevice2`) DrawIndexedPrimitive (+0x78) with
//! a triangle list (4) of transformed and lit vertices (3) and no flags. Callers:
//! `effects::windscreen_drops_top`, `effects::windscreen_drops_frame`.
//! Platform: the original always runs. In the standalone program the rasteriser draws
//! each triangle (`platform_3d::draw_triangles`) and the result is 0 (before 2026-10-19
//! the standalone skipped it).

use crate::gfx::windscreen_drop_batch::WINDSCREEN_DROP_BATCH;

/// The windscreen drops drawn: Direct3D's result, always 0 here.
pub fn draw_marks_windscreen_drops() -> u32 {
    crate::platform_3d::draw_indexed_bytes(
        &WINDSCREEN_DROP_BATCH.records,
        WINDSCREEN_DROP_BATCH.vertex_count.get(),
        &WINDSCREEN_DROP_BATCH.fans,
        WINDSCREEN_DROP_BATCH.index_count.get(),
    );
    0
}
