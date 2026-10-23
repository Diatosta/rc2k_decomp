//! `scene::clear_fog` (0x44FC19): no fog.
//!
//! In:  nothing
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! `dword_636610` = 0 and the 0x1000 bytes of `g_fogTable` = 0. (The original
//! clears the table first; the result is the same.)
//! One caller.
//! Self-test: the tables filled with another byte first, and put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, ECX, EDI, game_addr, game_write, rva};
use crate::screens::harness::Left;

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 1] = [
    (rva(0x63661C), 0x1000, 0x00), // g_fogTable
];

/// Cleared first.
const CLEARED_RVA: usize = rva(0x636610); // dword_636610

// ---- Rust version ----

pub unsafe fn clear_fog() -> Left {
    unsafe {
        game_write(CLEARED_RVA, 0u32);
        for (table, len, byte) in FILLS {
            std::ptr::write_bytes(game_addr(table) as *mut u8, byte, len);
        }
    }
    let (last, len, _) = FILLS[FILLS.len() - 1];
    Left::nothing()
        .with(EAX, 0x0)
        .with(ECX, 0)
        .with(EDI, (game_addr(last) + len) as u32)
}
