//! Finds a record by index within a section of a record table.
//!
//! Section 0 is the table's start; section N starts after the Nth record whose tag is 4. There
//! is no end-of-table check.

use crate::errors::BadInput;
use crate::records::stages_store::{RECORD_LEN, RecordRef, RecordTable, StageTables};

/// The record, with no end test; a table running past its block is refused.
pub fn find_record_in(
    stages: &StageTables,
    table: RecordTable,
    index: i32,
    section: u32,
) -> Result<RecordRef, BadInput> {
    let place = rc2k_core::files::records::find_record(&table.bytes(stages), index, section)?;
    let index = u32::try_from(place / RECORD_LEN as isize)
        .map_err(|_| BadInput("a record index before its table"))?;
    table.row(stages, index)
}

pub fn has_section(
    stages: &StageTables,
    table: RecordTable,
    section: u32,
    limit: u32,
) -> Result<bool, BadInput> {
    Ok(rc2k_core::files::records::has_section(
        &table.bytes(stages),
        section,
        limit as usize,
    ))
}
