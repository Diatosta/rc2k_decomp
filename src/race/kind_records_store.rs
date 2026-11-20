//! The course kinds' first and last records, indexed by kind (`physics::ground`,
//! `race::course_data`, `physics::find_course_record`). The course store follows
//! (`race::course_store`).

use crate::memory::cells::ZeroCells;
use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct KindRecords {
    /// Each kind's first record.
    pub first_records: [PlainDword; 256],
    /// Each kind's last record.
    pub last_records: [PlainDword; 259],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with its first values.
impl KindRecords {
    pub const fn new() -> Self {
        Self { ..ZeroCells::ZERO }
    }
}
