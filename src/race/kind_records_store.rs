//! The course kinds' first and last records, indexed by kind (`physics::ground`,
//! `race::course_data`, `physics::find_course_record`). The course store follows
//! (`race::course_store`).

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct KindRecords {
    /// Unused; only dead code touched it.
    pub unused_0: [PlainDword; 9],
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
        Self {
            unused_0: {
                let mut a: [PlainDword; 9] = ZeroCells::ZERO;
                a[1] = PlainDword::new(0xFFFFA000);
                a[2] = PlainDword::new(0x11000);
                a[3] = PlainDword::new(0xFFFFC000);
                a
            },
            ..ZeroCells::ZERO
        }
    }
}
