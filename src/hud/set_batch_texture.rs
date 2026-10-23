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
use crate::framework::{Regs, game_read, rva};
use crate::hud::batch::SHOWN_TEXTURE_RVA;

const SLOTS_RVA: usize = rva(0x5D1AE8); // g_kindFirstSlots

/// A call by place: 1 `set_texture`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::batch::texture_slot`
pub unsafe fn set_batch_texture(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let n = rc2k_core::hud::batch::texture_slot(unsafe { game_read(SHOWN_TEXTURE_RVA) })?;
    let mut r = Regs {
        eax: unsafe { game_read(SLOTS_RVA + 4 * n) },
        ..*entry
    };
    part(1, &mut r)?;
    Ok(*entry)
}

/// In the game: `set_texture`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::scene::set_texture::set_texture_in_game(r) };
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { set_batch_texture(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
