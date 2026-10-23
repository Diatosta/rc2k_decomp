//! `scene::reset_texture_tables` (0x42247B): forgets which textures the kinds need and where they are.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! `dword_5DC978` = 0; -1 throughout the kinds' two tables of textures needed
//! (`dword_5CFAE0`, `dword_5CEAE0`) and their first slots and first textures
//! (`g_kindFirstSlots`, `g_kindFirstTextures`), 0x400 dwords each.
//! Four callers.
//! Self-test: the tables filled with another byte first, and put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDI, Regs, game_addr, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 4] = [
    (rva(0x5CFAE0), 0x1000, 0xFF), // dword_5CFAE0
    (rva(0x5CEAE0), 0x1000, 0xFF), // dword_5CEAE0
    (rva(0x5D1AE8), 0x1000, 0xFF), // g_kindFirstSlots
    (rva(0x5D0AE8), 0x1000, 0xFF), // g_kindFirstTextures
];

/// Cleared first.
const CLEARED_RVA: usize = rva(0x5DC978); // dword_5DC978

// ---- Rust version ----

pub unsafe fn reset_texture_tables() -> Left {
    unsafe {
        game_write(CLEARED_RVA, 0u32);
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { reset_texture_tables() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
