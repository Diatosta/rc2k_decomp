//! `menu::clear_event_state` (0x486C2C): an event's state cleared at its start.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755950`, `dword_758714` and the 28 dwords `dword_758758` .. `dword_7587C4` set to
//! 0 (the results screens and the savegame read them; their meaning is not read). Callers:
//! the event loops (`sub_482382` and four more). Self-test: the variables filled with 0xAA,
//! put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

const FIRST_RVA: usize = rva(0x755950); // dword_755950
const SECOND_RVA: usize = rva(0x758714); // dword_758714
const RUN_RVA: usize = rva(0x758758); // dword_758758 .. dword_7587C4
const RUN_DWORDS: usize = 28;

fn regions() -> [Region; 3] {
    [
        at(FIRST_RVA),
        at(SECOND_RVA),
        (game_addr(RUN_RVA) as u32, RUN_DWORDS * 4),
    ]
}

pub fn clear_event_state() {
    for (addr, len) in regions() {
        unsafe { std::ptr::write_bytes(addr as *mut u8, 0, len) };
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    clear_event_state();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
