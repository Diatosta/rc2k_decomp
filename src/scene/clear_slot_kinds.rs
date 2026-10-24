//! `scene::clear_slot_kinds` (0x426AB7): no object slot has a kind.
//!
//! In:  nothing
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! The 0x100 dwords of `g_slotKinds` = 0. Two callers.

use crate::framework::{game_addr, rva};

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 1] = [
    (rva(0x5CCE7C), 0x400, 0x00), // g_slotKinds
];

// ---- Rust version ----

pub unsafe fn clear_slot_kinds() {
    unsafe {
        for (table, len, byte) in FILLS {
            std::ptr::write_bytes(game_addr(table) as *mut u8, byte, len);
        }
    }
}
