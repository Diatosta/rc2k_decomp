//! `scene::assign_kind_textures` (0x4256D8): every texture slot gets the texture of
//! the kind it is the first slot of.
//!
//! In:  nothing
//! Out: eax = kind 0's first slot, ebx = kind 0's first texture, ecx = 0xFFFFFFFF,
//!      edi = the end of the slots' table
//!
//! The 0x400 slots of `dword_5DA968` = -1; then for the kinds from 0x3FF down to 0:
//! the kind's first slot (`dword_5D1AE8`), unless it is -1, gets the kind's first
//! texture (`dword_5D0AE8`). So of two kinds with one slot the lower one's stays.
//! Six callers.
//! Refuses a first slot past the table.
//! Self-test: 8 tables by lot in the game's places, put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDI, Regs, game_addr};
use crate::scene::textures::{
    KIND_SLOT_RVA, KIND_TEXTURE_RVA, KINDS, SLOTS, WANTED_RVA, entry as table_entry, slice,
};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

pub unsafe fn assign_kind_textures() -> Result<Left, BadInput> {
    // Converted: `rc2k_core::scene::textures::assign_kind_textures`.
    unsafe {
        rc2k_core::scene::textures::assign_kind_textures(
            slice(KIND_SLOT_RVA, KINDS),
            slice(KIND_TEXTURE_RVA, KINDS),
            slice(WANTED_RVA, SLOTS),
        )?;
        Ok(Left::nothing()
            .with(EAX, table_entry(KIND_SLOT_RVA, 0))
            .with(EBX, table_entry(KIND_TEXTURE_RVA, 0))
            .with(ECX, u32::MAX)
            .with(EDI, game_addr(WANTED_RVA) as u32 + SLOTS * 4))
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { assign_kind_textures() }?)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
