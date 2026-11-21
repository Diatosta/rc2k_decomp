//! Clears the first six dwords (the force and the point) of each record of a table of 0x20-byte
//! records.

use crate::physics::tables_store::ForceRecord;

pub fn clear_records(records: &[ForceRecord]) {
    for record in records {
        for part in record.force.iter().chain(&record.point) {
            part.set(0.0);
        }
    }
}
