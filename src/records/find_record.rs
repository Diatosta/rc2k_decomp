//! `records::find_record` (0x47255D): finds a record by index within a section of a record table.
//!
//! In:  eax = index, ebx = section (0 = from the table start), edi = table
//! Out: edi = record pointer; eax, ebx, ecx and edx preserved
//! Section N starts after the Nth record whose tag is 4. There is no end-of-table check.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::bridge::{Raw, read_through};

// ---- Rust version ----

/// The record, as the original finds it (converted: `rc2k_core::files::records::find_record`, on
/// the table in game memory: no end test, as the original).
pub fn find_record_in(table: u32, index: i32, section: u32) -> u32 {
    let records = unsafe { Raw::new(table as usize as *const u8) };
    let place = read_through(rc2k_core::files::records::find_record(
        &records, index, section,
    ));
    table.wrapping_add(place as u32)
}

/// True if `table` has at least `section` section headers among its first `limit` records:
/// a bounded check that `find_record_in` will not run off the table (converted:
/// `rc2k_core::files::records::has_section`).
pub fn has_section(table: u32, section: u32, limit: u32) -> bool {
    let records = unsafe { Raw::new(table as usize as *const u8) };
    rc2k_core::files::records::has_section(&records, section, limit as usize)
}
