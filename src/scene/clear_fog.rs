//! `scene::clear_fog` (0x44FC19): no fog.
//!
//! In:  nothing
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! `dword_636610` = 0 and the 0x1000 bytes of `g_fogTable` = 0. (The original clears the table
//! first; the result is the same.) One caller.

use crate::scene::light_store::LIGHT_STATE;

// ---- game data ----

/// `g_fogTable`: 0x1000 bytes, cleared.
const FOG_BYTES: usize = 0x1000;

// ---- Rust version ----

pub unsafe fn clear_fog() {
    unsafe {
        LIGHT_STATE.fog_next.set_bits(0u32);
        std::ptr::write_bytes(LIGHT_STATE.fog_table[0].addr() as *mut u8, 0x00, FOG_BYTES);
    }
}
