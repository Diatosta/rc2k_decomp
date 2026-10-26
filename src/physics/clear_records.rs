//! `physics::clear_records` (0x43F3A6): ecx records of 0x20 bytes from edi, the first six
//! dwords of each 0 (`dec ecx; jnz` back to its own first instruction).
//!
//! In:  edi the first record, ecx their number
//! Out: edi past the last, ecx 0; the rest kept; ZF set (`dec`), not read (its callers
//!      return next, theirs go on with calls). Nothing is returned now
//!
//! Its loop goes back to its first byte: `.private_copy` (the whole function).
//! Callers: `physics::clear_forces` (0x614770, 0x17), `physics::clear_contacts` (0x614A50,
//! 7). Refuses (before anything) ecx 0 (the original would loop 2^32 times) or above 0x100.

use crate::errors::Refused;

/// The Rust version.
// Converted: `rc2k_core::physics::pushes::{record_count, CLEARED_DWORDS}`.
pub unsafe fn clear_records(at: u32, count: u32) -> Result<(), Refused> {
    use rc2k_core::physics::pushes::{CLEARED_DWORDS, record_count};
    let count = record_count(count, "a record count of 0 or above 0x100")?;
    for k in 0..count {
        for d in 0..CLEARED_DWORDS {
            crate::pointers::write_or_end::<u32>(
                at + 0x20 * k + 4 * d,
                0,
                "physics::clear_records's operand",
            );
        }
    }
    Ok(())
}
