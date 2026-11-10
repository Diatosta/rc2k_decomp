//! `hud::flush_batch` (0x4B74EF): unless the batch holds no indices, drawn by the device
//! (`d3d::draw_hud_batch`: `DrawIndexedPrimitive` through `g_d3dDevice`, the platform's) and
//! emptied.
//!
//! Callers: `hud::draw_hud` (the HUD's end), `hud::add_polygon`.

// Converted: `rc2k_core::hud::batch::flush_due`
pub fn flush_batch(hud: &Hud) {
    if !rc2k_core::hud::batch::flush_due(hud.block.index_count.get()) {
        return;
    }
    crate::d3d::draw_hud_batch::draw_hud_batch(hud);
    hud.block.vertex_count.set(0);
    hud.block.index_count.set(0);
}
use crate::hud::owner::Hud;
