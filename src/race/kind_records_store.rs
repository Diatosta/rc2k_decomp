//! The course kinds' first and last records, 0x5CC3C0 .. 0x5CCBF0: `g_kindFirstRecords` and
//! `g_kindLastRecords`, indexed by kind (`physics::ground`, `race::course_data`,
//! `physics::find_course_record`). The course store follows (`race::course_store`).
//! Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct KindRecords {
    /// 0x5CC3C0: IDA touches +0x1c (0x5CC3DC) only in code outside any function (0x421C63 .. 0x421D96, dead); the port never reaches it.
    pub unused_0: [PlainDword; 9],
    /// 0x5CC3E4: `g_kindFirstRecords`: each kind's first record (`physics::ground`, `race::course_data`, `physics::find_course_record`).
    pub first_records: [PlainDword; 256],
    /// 0x5CC7E4: `g_kindLastRecords`: each kind's last record (`physics::ground`, `race::course_data`).
    pub last_records: [PlainDword; 259],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
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

#[cfg(test)]
mod tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::race::owner::RACE.course.kind_records(),
        );
    }
}
