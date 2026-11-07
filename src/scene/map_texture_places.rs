//! `scene::map_texture_places` (0x42570D): for each texture's place, the slot that
//! shows it.
//!
//! The 0xFA0 dwords of `dword_5D3AE8` = -1; then for the slots from 0x3FF down to 0: the
//! texture the slot wants (`dword_5DA968`), if under 0x400, has a place (`dword_5D8968`), and
//! that place, if under 0xFA0, gets the slot's number. Of two slots with one place the lower
//! stays. Six callers. The registers above are the original's; a call by address goes through
//! `routes.rs`, which maps them.

use crate::scene::textures::with_cells;

// ---- Rust version ----

/// What the original leaves in eax, ecx and edi no caller reads (register liveness,
/// 2026-10-23: its five call sites).
pub fn map_texture_places(scene: &crate::scene::owner::Scene) {
    let textures = &scene.textures;
    // Converted: `rc2k_core::scene::textures::map_places`.
    with_cells(
        [
            &textures.places.slot_wanted,
            &textures.places.wanted_pictures,
            &textures.places.place_slot,
        ],
        |[wanted, pictures, places]| {
            rc2k_core::scene::textures::map_places(wanted, pictures, places)
        },
    );
}
