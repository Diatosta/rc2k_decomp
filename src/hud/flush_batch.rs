//! `hud::flush_batch` (0x4B74EF): unless the batch holds no indices, drawn by the device
//! (`sub_45A869`: `DrawIndexedPrimitive` through `g_d3dDevice`, the platform's) and
//! emptied.
//!
//! In:  nothing
//! Out: as the device's call leaves the registers when drawn; else kept
//!
//! Callers: `sub_4B1CD0` (the HUD's end), `hud::add_polygon`.

/// The Rust version (see the header).
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
