//! `scene::assign_kind_textures` (0x4256D8): every texture slot gets the texture of
//! the kind it is the first slot of.
//!
//! In:  nothing
//! Out: eax = kind 0's first slot, ebx = kind 0's first texture, ecx = 0xFFFFFFFF,
//!      edi = the end of the slots' table
//!
//! The 0x400 slots of `dword_5DA968` = -1; then for the kinds from 0x3FF down to 0: the kind's
//! first slot (`dword_5D1AE8`), unless it is -1, gets the kind's first texture
//! (`dword_5D0AE8`). So of two kinds with one slot the lower one's stays. Six callers. Refuses
//! a first slot past the table.

use crate::errors::BadInput;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::scene::textures::{SLOTS, WANTED_RVA, cells_mut, slice};

// ---- Rust version ----

/// What the original leaves in eax, ebx, ecx and edi no caller reads (register liveness,
/// 2026-10-23: its five call sites).
pub unsafe fn assign_kind_textures() -> Result<(), BadInput> {
    // Converted: `rc2k_core::scene::textures::assign_kind_textures`.
    unsafe {
        rc2k_core::scene::textures::assign_kind_textures(
            cells_mut(&KIND_TEXTURES.kind_first_slots),
            cells_mut(&KIND_TEXTURES.kind_first_textures),
            slice(WANTED_RVA, SLOTS),
        )?;
        Ok(())
    }
}
