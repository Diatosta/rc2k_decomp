//! Unless the batch holds no indices, drawn by the device (`d3d::draw_hud_batch`) and emptied.

pub fn flush_batch(part: &crate::scene::scratch_store::PartScratch, hud: &Hud) {
    if !rc2k_core::hud::batch::flush_due(hud.block.index_count.get()) {
        return;
    }
    crate::d3d::draw_hud_batch::draw_hud_batch(part, hud);
    hud.block.vertex_count.set(0);
    hud.block.index_count.set(0);
}
use crate::hud::owner::Hud;
