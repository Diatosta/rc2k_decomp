//! `race::permute_bits` (0x402848): edx:eax's bits picked by a table at ebx (a count
//! n, then n + 1 picks: bit 5 set eax's bit (b & 0x1F), else edx's) gathered into
//! esi:edi from the top down.
//!
//! In:  eax, edx = the bits, ebx = the table
//! Out: esi:edi = the result, ebx = past the table; the rest preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_402735` five times (the save's sealing;
//! none reads its flags). Shadow mode: the Rust version and the original, registers
//! compared (nothing written). Self-test: the game's five tables and tables of the
//! test's (counts 0, 0x3F, high bits set in the picks), values by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::race_shell::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { permute_bits(r) };
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
