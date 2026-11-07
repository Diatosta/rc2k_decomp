//! `records::find_record` (0x47255D): finds a record by index within a section of a record table.
//!
//! Takes the index, the section (0 = from the table start) and the table. Returns the record.
//! Section N starts after the Nth record whose tag is 4. There is no end-of-table check.

use crate::errors::BadInput;
use crate::records::stages_store::{RECORD_LEN, RecordRef, RecordTable};

// ---- Rust version ----

/// The record, as the original finds it (converted: `rc2k_core::files::records::find_record`, on
/// the table in game memory: no end test, as the original; a table running past its block
/// is refused).
pub fn find_record_in(table: RecordTable, index: i32, section: u32) -> Result<RecordRef, BadInput> {
    let place = rc2k_core::files::records::find_record(&table.bytes(), index, section)?;
    let index = u32::try_from(place / RECORD_LEN as isize)
        .map_err(|_| BadInput("a record index before its table"))?;
    table.row(index)
}

pub fn has_section(table: RecordTable, section: u32, limit: u32) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::records::has_section(
        &table.bytes(),
        section,
        limit as usize,
    ))
}
