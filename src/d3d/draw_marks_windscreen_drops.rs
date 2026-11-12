//! `d3d::draw_marks_windscreen_drops` (0x41ADB9): draws the marks' windscreen drops, an indexed list of
//! triangles.
//!
//! In:  nothing: the vertices at `WindscreenDrops.records`, their number in `WindscreenDrops.vertex_count`, the indices
//!      (16 bits) at `g_marksWindscreenDropFans`, their number in `WindscreenDrops.index_count`
//! The device's (`g_d3dDevice`, an `IDirect3DDevice2`) DrawIndexedPrimitive (+0x78) with
//! a triangle list (4) of transformed and lit vertices (3) and no flags. Callers:
//! `effects::windscreen_drops_top`, `effects::windscreen_drops_frame`.
//! Platform: the original always runs. In the standalone program the rasteriser draws
//! each triangle (`platform_3d::draw_triangles`) and the result is 0.

/// The windscreen drops drawn: Direct3D's result, always 0 here.
pub fn draw_marks_windscreen_drops(
    part: &crate::workers::PartScratch,
    gfx: &crate::gfx::owner::Gfx,
) -> u32 {
    let card = &gfx.card;
    use crate::gfx::windscreen_drop_batch::{FANS_LEN, RECORDS_LEN};
    crate::platform_3d::draw_indexed_bytes(
        part,
        &card.windscreen_drop_batch.records,
        RECORDS_LEN,
        card.windscreen_drop_batch.vertex_count.get(),
        &card.windscreen_drop_batch.fans,
        FANS_LEN,
        card.windscreen_drop_batch.index_count.get(),
    );
    0
}
