//! `d3d::draw_marks_windscreen_drops`: draws the marks' windscreen drops, an indexed list of
//! triangles of transformed and lit vertices with no flags. The vertices are at
//! `WindscreenDrops.records`, their number in `WindscreenDrops.vertex_count`, the indices (16 bits)
//! in the drops' fan table, their number in `WindscreenDrops.index_count`. The rasteriser draws
//! each triangle (`platform::direct3d::draw_triangles`); the result is 0.

/// Draws the windscreen drops' batch.
pub fn draw_marks_windscreen_drops(
    part: &crate::scene::workers::PartScratch,
    gfx: &crate::gfx::owner::Gfx,
) {
    let card = &gfx.card;
    use crate::gfx::windscreen_drop_batch::{FANS_LEN, RECORDS_LEN};
    crate::platform::direct3d::draw_indexed_bytes(
        part,
        &card.windscreen_drop_batch.records,
        RECORDS_LEN,
        card.windscreen_drop_batch.vertex_count.get(),
        &card.windscreen_drop_batch.fans,
        FANS_LEN,
        card.windscreen_drop_batch.index_count.get(),
    );
}
