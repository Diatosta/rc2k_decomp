//! `scene::clear_fog` (0x44FC19): no fog.
//!
//! In:  nothing
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! `dword_636610` = 0 and the 0x1000 bytes of `g_fogTable` = 0. (The original clears the table
//! first; the result is the same.) One caller.

use crate::scene::light_store::LIGHT_STATE;

// ---- game data ----

// ---- Rust version ----

pub fn clear_fog() {
    LIGHT_STATE.fog_next.set_bits(0u32);
    for cell in &LIGHT_STATE.fog_table {
        cell.set(0);
    }
}
