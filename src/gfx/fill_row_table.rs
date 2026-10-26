//! `gfx::fill_row_table` (0x40586A): fills the table of row addresses with a start address
//! and so many steps on from it.
//!
//! In:  eax = the first row's address, ebx = the bytes from one row to the next, ecx = rows
//! Out: every register preserved
//! The table is `dword_4F3FC4`, which has room for 602 rows. Four callers, the rescalers
//! and two picture functions not yet read.
//! Refuses a count of 0 (the original counts through all 32 bits) and one above the table's
//! room. The name is a reading.

use crate::errors::BadInput;
use crate::session::Dword;

// ---- game data ----

/// IDA's address of the row table.
pub const START: usize = 0x4F3FC4;
pub const ROW_TABLE_ROOM: u32 = 602;
/// The table's length.
pub const LEN: usize = 4 * ROW_TABLE_ROOM as usize;

/// `dword_4F3FC4`: the address of each row of a picture (602 of them), zero at the start.
pub static ROW_TABLE: [Dword; ROW_TABLE_ROOM as usize] =
    [const { Dword::new(0) }; ROW_TABLE_ROOM as usize];

/// Where the table is (`moved::MOVED`'s home for IDA 0x4F3FC4).
pub fn home() -> usize {
    ROW_TABLE.as_ptr() as usize
}

/// The address the table holds for a row.
pub fn row_address(row: u32) -> u32 {
    match ROW_TABLE.get(row as usize) {
        Some(cell) => cell.get(),
        None => crate::fatal::raise::end(
            "gfx::row_address",
            BadInput("a row past the row table's 602"),
        ),
    }
}

// ---- Rust version ----

pub unsafe fn fill_row_table(first: u32, step: u32, rows: u32) -> Result<(), BadInput> {
    if rows == 0 || rows > ROW_TABLE_ROOM {
        return Err(BadInput("a row count the row table has no room for"));
    }
    // Converted: `rc2k_core::gfx::pixels::row_table`.
    let table = ROW_TABLE.as_ptr() as *mut u32;
    for (row, address) in rc2k_core::gfx::pixels::row_table(first, step, rows as usize)
        .into_iter()
        .enumerate()
    {
        unsafe { table.add(row).write_unaligned(address) };
    }
    Ok(())
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&ROW_TABLE, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), home());
        assert_eq!(crate::moved::place(START + LEN - 4), ROW_TABLE[601].addr());
    }
}
