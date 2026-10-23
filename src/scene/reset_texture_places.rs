//! `scene::reset_texture_places` (0x423FE1): no texture has a place.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `dword_5D9968` and `dword_5D8968` (each texture's place), 0x400
//! dwords each.
//! Two callers.
//! Self-test: the tables filled with another byte first, and put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDI, Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 2] = [
    (rva(0x5D9968), 0x1000, 0xFF), // dword_5D9968
    (rva(0x5D8968), 0x1000, 0xFF), // dword_5D8968
];

// ---- Rust version ----

pub unsafe fn reset_texture_places() -> Left {
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { reset_texture_places() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
