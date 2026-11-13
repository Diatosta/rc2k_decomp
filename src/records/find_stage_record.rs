//! In a record table (0xA4-byte records, +0 the tag), past the `section`'s start (the
//! section-th record of tag 4, none for 0), the `number`-th record of tag 0 (a stage, counted
//! from 1): the time trial results page.
//!
//! Called by `screens::draw_stage_results`. There is no end test, so a number of 0 (or one past
//! the stages) would run on: refused, as is a search that finds nothing within 0x1000 records.

use crate::errors::Refused;
use crate::records::stages_store::{RECORD_LEN, RecordRef, RecordTable, StageTables};

/// The record (refused where the search would run away).
pub fn find_stage_record(
    stages: &StageTables,
    table: RecordTable,
    section: u32,
    number: u32,
) -> Result<RecordRef, Refused> {
    let place =
        rc2k_core::files::records::find_stage_record(&table.bytes(stages), section, number)?;
    Ok(table.row(stages, (place / RECORD_LEN as usize) as u32)?)
}
