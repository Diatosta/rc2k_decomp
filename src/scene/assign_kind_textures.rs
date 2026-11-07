//! `scene::assign_kind_textures` (0x4256D8): every texture slot gets the texture of
//! the kind it is the first slot of.
//!
//! Out: eax = kind 0's first slot, ebx = kind 0's first texture, ecx = 0xFFFFFFFF,
//!      edi = the end of the slots' table
//!
//! The 0x400 slots of `dword_5DA968` = -1; then for the kinds from 0x3FF down to 0: the kind's
//! first slot (`dword_5D1AE8`), unless it is -1, gets the kind's first texture
//! (`dword_5D0AE8`). So of two kinds with one slot the lower one's stays. Six callers. Refuses
//! a first slot past the table.

use crate::errors::BadInput;
use crate::scene::textures::with_cells;

// ---- Rust version ----

/// What the original leaves in eax, ebx, ecx and edi no caller reads (register liveness,
/// 2026-10-23: its five call sites).
pub fn assign_kind_textures(scene: &crate::scene::owner::Scene) -> Result<(), BadInput> {
    let scene_textures = &scene.textures;
    // Converted: `rc2k_core::scene::textures::assign_kind_textures`.
    with_cells(
        [
            &scene_textures.kinds.kind_first_slots,
            &scene_textures.kinds.kind_first_textures,
            &scene_textures.places.slot_wanted,
        ],
        |[slots, textures, wanted]| {
            rc2k_core::scene::textures::assign_kind_textures(slots, textures, wanted)
        },
    )?;
    Ok(())
}
