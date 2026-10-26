//! `records::find_stage_record` (0x468289): in a record table (0xA4-byte records, +0 the
//! tag), past section ebx's start (the ebx-th record of tag 4, none for 0), the eax-th record
//! of tag 0 (a stage; counted from 1). (Stream A, part F1: the time trial results page.)
//!
//! In:  edi = the table, ebx = the section, eax = the stage's number
//! Out: edi = the record; eax, ebx, ecx kept (the rest untouched)
//!
//! Caller: `screens::draw_stage_results` (0x48EB91). The original has no end test: a number
//! of 0 (or one past the stages) runs on through memory. Refused, as is a search that finds
//! nothing within 0x1000 records.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::pointers::GamePtr;

/// The record, as the original finds it (refused where it would run away). Converted:
/// `rc2k_core::files::records::find_stage_record`.
pub fn find_stage_record(table: u32, section: u32, number: u32) -> Result<u32, Refused> {
    let records = GamePtr::new(table).bytes("a record table")?;
    let place = rc2k_core::files::records::find_stage_record(&records, section, number)?;
    Ok(table.wrapping_add(place as u32))
}
