//! `hud::set_batch_texture` (0x45A855): the device's texture set to the batch's
//! (`g_kindFirstSlots`[`dword_6D14FE`], `scene::set_texture`).
//!
//! In:  nothing
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `hud::add_polygon`. The texture call through `part` (1 with eax the texture),
//! in the game its Rust version. Refuses a texture past `g_kindFirstSlots` (0x400 dwords; the HUD asks for 0xC0 .. 0xDC), before
//! anything. Shadow mode runs the original alone (it tells the device). Self-test: a copy
//! whose call goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::rva;
use crate::hud::batch::SHOWN_TEXTURE_RVA;
use crate::hud::store::game_read;

const SLOTS_RVA: usize = rva(0x5D1AE8); // g_kindFirstSlots

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::batch::texture_slot`
pub fn set_batch_texture() -> Result<(), Refused> {
    let n = rc2k_core::hud::batch::texture_slot(unsafe { game_read(SHOWN_TEXTURE_RVA) })?;
    crate::scene::set_texture::set_texture(unsafe { game_read(SLOTS_RVA + 4 * n) });
    Ok(())
}
