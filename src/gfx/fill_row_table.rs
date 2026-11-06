//! `gfx::fill_row_table` (0x40586A): fills the table of row addresses with a start address
//! and so many steps on from it.
//!
//! In:  eax = the first row's address, ebx = the bytes from one row to the next, ecx = rows
//! Out: every register preserved
//! The table is `dword_4F3FC4`, which has room for 602 rows. Four callers, the rescalers
//! and two picture functions not yet read.
//! Refuses a count of 0 (the original counts through all 32 bits) and one above the table's
//! room. The name is a reading.
//! Since phase 12 (2026-10-27) the decoded picture is a Rust buffer (`gfx::picture`) and the
//! table holds each row's offset in it: its callers give 0 for the first row.

use crate::errors::BadInput;

// ---- game data ----

/// IDA's address of the row table.
pub const START: usize = 0x4F3FC4;
pub const ROW_TABLE_ROOM: u32 = 602;
/// The table's length.
pub const LEN: usize = 4 * ROW_TABLE_ROOM as usize;

/// Where the table is (`moved::MOVED`'s home for IDA 0x4F3FC4).
#[cfg(test)]
pub fn home() -> usize {
    (crate::pointers::game_address(crate::gfx::owner::gfx().drawing.row_table.as_ptr())) as usize
}

/// The address the table holds for a row.
pub fn row_address(row: u32) -> u32 {
    match crate::gfx::owner::gfx().drawing.row_table.get(row as usize) {
        Some(cell) => cell.get(),
        None => crate::fatal::raise::end(
            "gfx::row_address",
            BadInput("a row past the row table's 602"),
        ),
    }
}

// ---- Rust version ----

pub fn fill_row_table(first: u32, step: u32, rows: u32) -> Result<(), BadInput> {
    if rows == 0 || rows > ROW_TABLE_ROOM {
        return Err(BadInput("a row count the row table has no room for"));
    }
    // Converted: `rc2k_core::gfx::pixels::row_table`.
    for (cell, address) in
        crate::gfx::owner::gfx()
            .drawing
            .row_table
            .iter()
            .zip(rc2k_core::gfx::pixels::row_table(
                first,
                step,
                rows as usize,
            ))
    {
        cell.set(address);
    }
    Ok(())
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::gfx::owner::gfx().drawing.row_table,
            START,
            START + LEN,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), home());
        assert_eq!(
            crate::moved::place(START + LEN - 4),
            crate::gfx::owner::gfx().drawing.row_table[601].addr() as usize
        );
    }
}
