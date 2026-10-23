//! `hud::draw_hud` (0x4B1CD0): the race's HUD: `hud::set_batch_depth`; the batch empty and
//! without a texture (`dword_75C038` -1); the record `unk_6D13FE`'s times in tenths
//! (`hud::tenths`); by its +0xF4 `hud::layout_1` (1), `hud::layout_2` (2) or
//! `hud::layout_0`; then `hud::flush_batch`.
//!
//! In:  nothing
//! Out: ebp the record; the rest as the flush leaves them
//!
//! Caller: `sub_45A893` (0x45A8F0). Hooked through a private copy (a call at the entry).
//! The depth and tenths are their Rust versions; the layouts and the flush through `part`
//! (1 / 2 / 3 the layouts 0 / 1 / 2, 4 the flush), in the game their Rust versions. Shadow
//! mode runs the original alone (it draws). Self-test: a copy whose layouts and flush go
//! to stand-ins, against the Rust version, every layout.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_write, rva};
use crate::hud::batch::{BATCH_TEXTURE_RVA, INDEX_COUNT_RVA, VERTEX_COUNT_RVA};

pub const RECORD_RVA: usize = rva(0x6D13FE); // unk_6D13FE

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::batch::{Layout, NO_TEXTURE}`
pub unsafe fn draw_hud(entry: &Regs) -> Result<Regs, Refused> {
    use rc2k_core::hud::batch;
    let mut g = *entry;
    unsafe {
        crate::hud::set_batch_depth::set_batch_depth();
        game_write(BATCH_TEXTURE_RVA, batch::NO_TEXTURE);
        game_write(VERTEX_COUNT_RVA, 0u32);
        game_write(INDEX_COUNT_RVA, 0u32);
        g.ebp = game_addr(RECORD_RVA) as u32;
        g = crate::hud::tenths::tenths(&g);
        let layout = match batch::Layout::of(((g.ebp + 0xF4) as *const u32).read_unaligned()) {
            batch::Layout::Single => 1,
            batch::Layout::One => 2,
            batch::Layout::Two => 3,
        };
        crate::hud::draw_hud::game_part(layout, &mut g)?;
        crate::hud::draw_hud::game_part(4, &mut g)?;
    }
    Ok(g)
}

/// In the game: the layouts' and the flush's Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::*;
    unsafe {
        *r = match place {
            1 => layout_0::layout_0(r, &mut layout_0::game_part)?,
            2 => layout_1::layout_1(r, &mut layout_1::game_part)?,
            3 => layout_2::layout_2(r, &mut layout_2::game_part)?,
            _ => flush_batch::flush_batch(r, &mut flush_batch::game_part)?,
        };
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_hud(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
