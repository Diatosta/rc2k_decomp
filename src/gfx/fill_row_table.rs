//! `gfx::fill_row_table` (0x40586A): fills the table of row addresses with a start address
//! and so many steps on from it.
//!
//! In:  eax = the first row's address, ebx = the bytes from one row to the next, ecx = rows
//! Out: every register preserved
//! The table is `dword_4F3FC4`, which has room for 602 rows. Four callers, the rescalers
//! and two picture functions not yet read.
//! Refuses a count of 0 (the original counts through all 32 bits) and one above the table's
//! room. The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

pub const ROW_TABLE_RVA: usize = rva(0x4F3FC4); // dword_4F3FC4
pub const ROW_TABLE_ROOM: u32 = 602;

/// The address the table holds for a row.
pub unsafe fn row_address(row: u32) -> u32 {
    unsafe {
        (game_addr(ROW_TABLE_RVA) as *const u32)
            .add(row as usize)
            .read_unaligned()
    }
}

// ---- Rust version ----

pub unsafe fn fill_row_table(first: u32, step: u32, rows: u32) -> Result<(), BadInput> {
    if rows == 0 || rows > ROW_TABLE_ROOM {
        return Err(BadInput("a row count the row table has no room for"));
    }
    // Converted: `rc2k_core::gfx::pixels::row_table`.
    let table = game_addr(ROW_TABLE_RVA) as *mut u32;
    for (row, address) in rc2k_core::gfx::pixels::row_table(first, step, rows as usize)
        .into_iter()
        .enumerate()
    {
        unsafe { table.add(row).write_unaligned(address) };
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { fill_row_table(r.eax, r.ebx, r.ecx) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
