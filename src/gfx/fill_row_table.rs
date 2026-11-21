//! Fills the table of row addresses with a start address and so many steps on from it: the first
//! row's address, the bytes from one row to the next, the rows. The table is `row_table`, which has
//! room for 602 rows. Refuses a count of 0 and one above the table's room. The decoded picture is a
//! Rust buffer (`gfx::picture`) and the table holds each row's offset in it: its callers give 0 for
//! the first row.

use crate::errors::BadInput;

pub(super) const ROW_TABLE_ROOM: u32 = 602;

/// The address the table holds for a row.
pub(super) fn row_address(row: u32, gfx: &crate::gfx::owner::Gfx) -> u32 {
    match gfx.drawing.row_table.get(row as usize) {
        Some(cell) => cell.get(),
        None => crate::fatal::raise::end(
            "gfx::row_address",
            BadInput("a row past the row table's 602"),
        ),
    }
}

pub(super) fn fill_row_table(
    first: u32,
    step: u32,
    rows: u32,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), BadInput> {
    if rows == 0 || rows > ROW_TABLE_ROOM {
        return Err(BadInput("a row count the row table has no room for"));
    }
    for (cell, address) in gfx
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
