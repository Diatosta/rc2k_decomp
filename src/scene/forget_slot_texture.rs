//! `scene::forget_slot_texture` (0x44EB66): slot eax's loaded texture forgotten
//! (`g_slotLoadedTextures[eax]` -1), so that it is loaded again.
//!
//! In:  eax the slot
//! Out: every register kept
//!
//! 12 callers (among them the marks' snow textures). Refuses a slot past 0x3FF (the table
//! is 0x400 dwords). Self-test: the original and the Rust version on slots 0, 7 and 0x3FF,
//! the refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const LOADED_RVA: usize = rva(0x5DB968); // g_slotLoadedTextures
const SLOTS: u32 = 0x400;

/// The Rust version (see the header).
pub unsafe fn forget_slot_texture(slot: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::scene::textures::forget_slot_texture`.
    let loaded = unsafe { crate::scene::textures::slice(LOADED_RVA, SLOTS) };
    Ok(rc2k_core::scene::textures::forget_slot_texture(
        loaded, slot,
    )?)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { forget_slot_texture(r.eax) }?;
    Ok(left_of(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
