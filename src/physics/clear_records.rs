//! `physics::clear_records` (0x43F3A6): clears the first six dwords of each of a table of records
//! of 0x20 bytes.
//!
//! Takes: the first record, their number
//!
//! Callers: `physics::clear_forces` (`forces`, 0x17 records), `physics::clear_contacts`
//! (`contacts`, 7). Refuses (before anything) no records (the original would loop 2^32 times)
//! or more than 0x100.

use crate::errors::Refused;

/// `clear_records_at` on the store's own place.
pub fn clear_records(at: &dyn crate::pointers::Store, count: u32) -> Result<(), Refused> {
    clear_records_at(crate::pointers::GamePtr::of_store(at), count)
}

// Converted: `rc2k_core::physics::pushes::{record_count, CLEARED_DWORDS}`.
pub fn clear_records_at(at: crate::pointers::GamePtr, count: u32) -> Result<(), Refused> {
    use rc2k_core::physics::pushes::{CLEARED_DWORDS, record_count};
    let count = record_count(count, "a record count of 0 or above 0x100")?;
    for k in 0..count {
        for d in 0..CLEARED_DWORDS {
            at.offset(0x20 * k + 4 * d)
                .write::<u32>(0, "physics::clear_records's operand")
                .unwrap_or_else(|e| crate::fatal::raise::end("physics::clear_records", e));
        }
    }
    Ok(())
}
