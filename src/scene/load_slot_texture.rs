//! `scene::load_slot_texture` (0x425ADE): gives a texture slot the texture it wants,
//! if it holds another.
//!
//! In:  ecx = the slot
//! Out: every register preserved
//!
//! Nothing if the slot wants none (`dword_5DA968[slot]` is -1) or holds what it wants
//! (`dword_5DB968[slot]`). Else `scene::load_texture` (0x44E778, which is only
//! `sub_41A8F9` of the Direct3D layer: not ported) with eax = the texture and ebx =
//! the slot, `dword_6415F8` + 1, and the slot holds it.
//! One caller, `scene::load_slot_textures`.
//! Refuses a slot past the table.
//! Shadow mode runs the original alone and checks what the slot holds afterwards.
//! Self-test: a redirected copy whose loading notes its registers.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::scene::textures::{LOADED_RVA, LOADS_RVA, WANTED_RVA, entry as slot_entry};

// ---- game data ----

const LOAD_RVA: usize = rva(0x44E778); // scene::load_texture

// ---- Rust version ----

/// `load`: what stands for `scene::load_texture`, given the texture and the slot.
// Converted: `rc2k_core::scene::textures::slot_load`
pub unsafe fn load_slot_texture(slot: u32, load: &mut dyn FnMut(u32, u32)) -> Result<(), BadInput> {
    unsafe {
        let Some(wanted) = rc2k_core::scene::textures::slot_load(
            slot,
            || slot_entry(WANTED_RVA, slot),
            || slot_entry(LOADED_RVA, slot),
        )?
        else {
            return Ok(());
        };
        load(wanted, slot);
        game_write(LOADS_RVA, game_read::<u32>(LOADS_RVA).wrapping_add(1));
        game_write(LOADED_RVA + slot as usize * 4, wanted);
        Ok(())
    }
}

/// With the game's loading, entered with the registers the original has there.
pub unsafe fn load_slot_texture_in_game(slot: u32, entry_regs: &Regs) -> Result<(), BadInput> {
    unsafe {
        load_slot_texture(slot, &mut |texture, slot| {
            let mut r = Regs {
                eax: texture,
                ebx: slot,
                ecx: slot,
                ..*entry_regs
            };
            crate::calls::call(LOAD_RVA, &mut r);
        })
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry_regs = *r;
    unsafe { load_slot_texture_in_game(r.ecx, &entry_regs) }?;
    Ok(())
}
