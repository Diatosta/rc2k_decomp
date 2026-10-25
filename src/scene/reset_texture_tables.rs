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

/// What is filled, in this order: where, how many bytes, with which byte.
fn fills() -> [(usize, usize, u8); 4] {
    [
        (OBJECT_TABLES.video_levels[0].addr(), 0x1000, 0xFF), // dword_5CFAE0
        (OBJECT_TABLES.system_levels[0].addr(), 0x1000, 0xFF), // dword_5CEAE0
        (KIND_TEXTURES.kind_first_slots[0].addr(), 0x1000, 0xFF), // g_kindFirstSlots
        (KIND_TEXTURES.kind_first_textures[0].addr(), 0x1000, 0xFF), // g_kindFirstTextures
    ]
}

// ---- Rust version ----

pub unsafe fn reset_texture_tables() {
    unsafe {
        TEXTURE_PLACES.kind_slot_bytes.set(0); // dword_5DC978, cleared first
        for (table, len, byte) in fills() {
            std::ptr::write_bytes(table as *mut u8, byte, len);
        }
    }
}
