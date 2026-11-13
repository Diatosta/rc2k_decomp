//! `physics::clear_records` (0x43F3A6): clears the first six dwords of each of a table of records
//! of 0x20 bytes: the force and the point of each force record.
//!
//! Takes: the table
//!
//! Callers: `physics::clear_forces` (`forces`, 0x17 records), `physics::clear_contacts`
//! (`contacts`, 7).

use crate::physics::tables_store::ForceRecord;

pub fn clear_records(records: &[ForceRecord]) {
    for record in records {
        for part in record.force.iter().chain(&record.point) {
            part.set_bits(0);
        }
    }
}
