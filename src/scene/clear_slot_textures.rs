//! `scene::clear_slot_textures` (0x4256A8): no texture slot holds or wants a texture.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `g_slotLoadedTextures` and `g_slotWantedTextures`, 0x400 dwords
//! each, so that every slot is loaded anew.
//! One caller, `scene::reload_slot_textures`.
//! Self-test: the tables filled with another byte first, and put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, ECX, EDI, game_addr, rva};
use crate::screens::harness::Left;

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 2] = [
    (rva(0x5DB968), 0x1000, 0xFF), // g_slotLoadedTextures
    (rva(0x5DA968), 0x1000, 0xFF), // g_slotWantedTextures
];

// ---- Rust version ----

pub unsafe fn clear_slot_textures() -> Left {
    unsafe {
        for (table, len, byte) in FILLS {
            std::ptr::write_bytes(game_addr(table) as *mut u8, byte, len);
        }
    }
    let (last, len, _) = FILLS[FILLS.len() - 1];
    Left::nothing()
        .with(EAX, 0xffffffff)
        .with(ECX, 0)
        .with(EDI, (game_addr(last) + len) as u32)
}
