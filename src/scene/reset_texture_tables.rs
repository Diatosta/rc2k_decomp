//! `scene::reset_texture_tables` (0x42247B): forgets which textures the kinds need and where they are.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! `dword_5DC978` = 0; -1 throughout the kinds' two tables of textures needed (`dword_5CFAE0`,
//! `dword_5CEAE0`) and their first slots and first textures (`g_kindFirstSlots`,
//! `g_kindFirstTextures`), 0x400 dwords each. Four callers.

use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::scene::texture_places_store::TEXTURE_PLACES;

// ---- game data ----

// ---- Rust version ----

pub fn reset_texture_tables() {
    TEXTURE_PLACES.kind_slot_bytes.set(0); // dword_5DC978, cleared first
    // dword_5CFAE0, dword_5CEAE0, g_kindFirstSlots, g_kindFirstTextures: 0x1000 bytes each.
    for table in [
        &OBJECT_TABLES.video_levels,
        &OBJECT_TABLES.system_levels,
        &KIND_TEXTURES.kind_first_slots,
        &KIND_TEXTURES.kind_first_textures,
    ] {
        for cell in table {
            cell.set(u32::MAX);
        }
    }
}
