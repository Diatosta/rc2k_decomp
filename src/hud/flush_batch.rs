//! `hud::flush_batch` (0x4B74EF): unless the batch holds no indices, drawn by the device
//! (`sub_45A869`: `DrawIndexedPrimitive` through `g_d3dDevice`, the platform's) and
//! emptied.
//!
//! In:  nothing
//! Out: as the device's call leaves the registers when drawn; else kept
//!
//! Callers: `sub_4B1CD0` (the HUD's end), `hud::add_polygon`. The device through `part`
//! (1), in the game the original's call (section 15). Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose device call goes to a stand-in, against the Rust
//! version, the batch empty and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, game_write};
use crate::hud::batch::{INDEX_COUNT_RVA, VERTEX_COUNT_RVA};

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::batch::flush_due`
pub fn flush_batch() {
    unsafe {
        if !rc2k_core::hud::batch::flush_due(game_read(INDEX_COUNT_RVA)) {
            return;
        }
        crate::d3d::draw_hud_batch::draw_hud_batch();
        game_write(VERTEX_COUNT_RVA, 0u32);
        game_write(INDEX_COUNT_RVA, 0u32);
    }
}
