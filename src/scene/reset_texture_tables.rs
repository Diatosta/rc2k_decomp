//! `scene::reset_texture_tables` (0x42247B): forgets which textures the kinds need and where they are.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! `dword_5DC978` = 0; -1 throughout the kinds' two tables of textures needed (`dword_5CFAE0`,
//! `dword_5CEAE0`) and their first slots and first textures (`g_kindFirstSlots`,
//! `g_kindFirstTextures`), 0x400 dwords each. Four callers.

use crate::framework::{game_addr, game_write, rva};
use crate::scene::kind_texture_store::KIND_TEXTURES;

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
fn fills() -> [(usize, usize, u8); 4] {
    [
        (game_addr(rva(0x5CFAE0)), 0x1000, 0xFF), // dword_5CFAE0
        (game_addr(rva(0x5CEAE0)), 0x1000, 0xFF), // dword_5CEAE0
        (KIND_TEXTURES.kind_first_slots[0].addr(), 0x1000, 0xFF), // g_kindFirstSlots
        (KIND_TEXTURES.kind_first_textures[0].addr(), 0x1000, 0xFF), // g_kindFirstTextures
    ]
}

/// Cleared first.
const CLEARED_RVA: usize = rva(0x5DC978); // dword_5DC978

// ---- Rust version ----

pub unsafe fn reset_texture_tables() {
    unsafe {
        game_write(CLEARED_RVA, 0u32);
        for (table, len, byte) in fills() {
            std::ptr::write_bytes(table as *mut u8, byte, len);
        }
    }
}
