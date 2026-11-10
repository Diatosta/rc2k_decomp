//! `physics::clear_records` (0x43F3A6): ecx records of 0x20 bytes from edi, the first six
//! dwords of each 0 (`dec ecx; jnz` back to its own first instruction).
//!
//! Takes: the first record, their number
//!
//! Its loop goes back to its first byte: `.private_copy` (the whole function).
//! Callers: `physics::clear_forces` (0x614770, 0x17), `physics::clear_contacts` (0x614A50,
//! 7). Refuses (before anything) ecx 0 (the original would loop 2^32 times) or above 0x100.

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
