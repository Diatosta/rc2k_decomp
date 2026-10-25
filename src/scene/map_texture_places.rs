//! `scene::map_texture_places` (0x42570D): for each texture's place, the slot that
//! shows it.
//!
//! In:  nothing
//! Out: eax = what was last looked at, ecx = 0xFFFFFFFF, edi = the end of the table
//!
//! The 0xFA0 dwords of `dword_5D3AE8` = -1; then for the slots from 0x3FF down to 0: the
//! texture the slot wants (`dword_5DA968`), if under 0x400, has a place (`dword_5D8968`), and
//! that place, if under 0xFA0, gets the slot's number. Of two slots with one place the lower
//! stays. Six callers. The registers above are the original's; a call by address goes through
//! `routes.rs`, which maps them.

use crate::scene::texture_places_store::TEXTURE_PLACES;
use crate::scene::textures::cells_mut;

// ---- Rust version ----

/// What the original leaves in eax, ecx and edi no caller reads (register liveness,
/// 2026-10-23: its five call sites).
pub unsafe fn map_texture_places() {
    // Converted: `rc2k_core::scene::textures::map_places`.
    unsafe {
        rc2k_core::scene::textures::map_places(
            cells_mut(&TEXTURE_PLACES.slot_wanted),
            cells_mut(&TEXTURE_PLACES.wanted_pictures),
            cells_mut(&TEXTURE_PLACES.place_slot),
        );
    }
}
