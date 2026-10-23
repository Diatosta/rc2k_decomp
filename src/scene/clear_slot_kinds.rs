//! `scene::clear_slot_kinds` (0x426AB7): no object slot has a kind.
//!
//! In:  nothing
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! The 0x100 dwords of `g_slotKinds` = 0.
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
const FILLS: [(usize, usize, u8); 1] = [
    (rva(0x5CCE7C), 0x400, 0x00), // g_slotKinds
];

// ---- Rust version ----

pub unsafe fn clear_slot_kinds() -> Left {
    unsafe {
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { clear_slot_kinds() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
