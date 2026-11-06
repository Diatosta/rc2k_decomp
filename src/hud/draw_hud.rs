//! `hud::draw_hud` (0x4B1CD0): the race's HUD: `hud::set_batch_depth`; the batch empty and
//! without a texture (`dword_75C038` -1); the record `unk_6D13FE`'s times in tenths
//! (`hud::tenths`); by its +0xF4 `hud::layout_1` (1), `hud::layout_2` (2) or
//! `hud::layout_0`; then `hud::flush_batch`.
//!
//! In:  nothing
//! Out: ebp the record; the rest as the flush leaves them
//!
//! Caller: `sub_45A893` (0x45A8F0).

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::pointers::GamePtr;
use crate::race::panel_store::PANEL;

/// The Rust version (see the header). What the original leaves in the registers is read
/// by none of its callers (register liveness, 2026-10-23).
// Converted: `rc2k_core::hud::batch::{Layout, NO_TEXTURE}`
pub fn draw_hud(hud: &Hud) -> Result<(), Refused> {
    use crate::hud::*;
    use rc2k_core::hud::batch;
    {
        set_batch_depth::set_batch_depth(hud);
        hud.block.batch_texture.set(batch::NO_TEXTURE);
        hud.block.vertex_count.set(0);
        hud.block.index_count.set(0);
        let record = GamePtr::to(&crate::race::groups_store::PANEL_AND_OVERLAY).offset(2);
        tenths::tenths(hud, record)?;
        match batch::Layout::of(PANEL.players.get()) {
            batch::Layout::Single => layout_0::layout_0(hud, record)?,
            batch::Layout::One => layout_1::layout_1(hud, record)?,
            batch::Layout::Two => layout_2::layout_2(hud, record)?,
        }
        flush_batch::flush_batch(hud);
    }
    Ok(())
}
