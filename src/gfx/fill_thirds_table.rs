//! `gfx::fill_thirds_table` (0x405580): the table of n / 3 for n = 0 .. 0x2FF.
//!
//! In:  nothing
//! Out: every register preserved
//! 0x300 bytes at `byte_4F4B30`: the mean of three components by their sum. One caller,
//! `start`. Who reads the table has not been looked up.

use crate::errors::Refused;
use crate::framework::{Regs, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

const TABLE_RVA: usize = rva(0x4F4B30); // byte_4F4B30
const TABLE_LEN: usize = 0x300;

// ---- Rust version ----

pub unsafe fn fill_thirds_table() {
    // Converted: `rc2k_core::gfx::pixels::thirds_table`.
    let table: [u8; TABLE_LEN] = rc2k_core::gfx::pixels::thirds_table();
    unsafe { game_write(TABLE_RVA, table) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { fill_thirds_table() };
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
