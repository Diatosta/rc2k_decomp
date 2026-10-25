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
use crate::framework::rva;
use crate::hud::store::HUD;
use crate::store::game_addr;

pub const RECORD_RVA: usize = rva(0x6D13FE); // unk_6D13FE

/// The Rust version (see the header). What the original leaves in the registers is read
/// by none of its callers (register liveness, 2026-10-23).
// Converted: `rc2k_core::hud::batch::{Layout, NO_TEXTURE}`
pub unsafe fn draw_hud() -> Result<(), Refused> {
    use crate::hud::*;
    use rc2k_core::hud::batch;
    unsafe {
        set_batch_depth::set_batch_depth();
        HUD.batch_texture.set(batch::NO_TEXTURE);
        HUD.vertex_count.set(0);
        HUD.index_count.set(0);
        let record = game_addr(RECORD_RVA) as u32;
        tenths::tenths(record);
        match batch::Layout::of(((record + 0xF4) as *const u32).read_unaligned()) {
            batch::Layout::Single => layout_0::layout_0(record)?,
            batch::Layout::One => layout_1::layout_1(record)?,
            batch::Layout::Two => layout_2::layout_2(record)?,
        }
        flush_batch::flush_batch();
    }
    Ok(())
}
