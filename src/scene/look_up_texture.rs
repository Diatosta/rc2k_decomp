//! `scene::look_up_texture` (0x44E668): `g_polygonTexture` for the texture word
//! `dword_6240C4` (stream A, part R3): -1 none; with the top bit a place
//! (`dword_5D3AE8[]`, below 0xFA0); else a kind's first slot (`g_kindFirstSlots[]`, below
//! 0x400); a slot not below `g_textureSlotCount` (or none loaded) none (0xFFFFFC43).
//!
//! In:  nothing (eax is not read: its caller 0x45FF6C loads one it does not use)
//! Out: eax as it leaves it (the word, or the slot); everything else kept
//!
//! Callers: 0x44CCB8, `race::flush_texture_batch`.

use crate::framework::{game_read, rva};
use crate::scene::draw_store::DRAW;
const PLACES_RVA: usize = rva(0x5D3AE8); // dword_5D3AE8
const FIRST_SLOTS_RVA: usize = rva(0x5D1AE8); // g_kindFirstSlots
const SLOT_COUNT_RVA: usize = rva(0x5DC970); // g_textureSlotCount

/// The Rust version: what it leaves in eax.
pub unsafe fn look_up_texture() -> u32 {
    // Converted: `rc2k_core::scene::textures::look_up`.
    unsafe {
        let found = rc2k_core::scene::textures::look_up(
            DRAW.texture_page.bits(),
            |p| game_read(PLACES_RVA + 4 * p),
            |k| game_read(FIRST_SLOTS_RVA + 4 * k),
            || game_read(SLOT_COUNT_RVA),
        );
        DRAW.polygon_texture.set_bits(found.texture);
        found.eax
    }
}
