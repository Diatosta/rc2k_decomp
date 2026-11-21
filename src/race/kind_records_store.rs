//! The course kinds' first and last records, indexed by kind (`physics::ground`,
//! `race::course_data`, `physics::find_course_record`). A plain record (`session::DrawPlain`):
//! written on the game's thread when a stage's records are grouped, read there and, as a copy
//! made when a drawing scope opens, by the drawing parts. The course store follows
//! (`race::course_store`).

use crate::memory::session::{DrawPlain, PlainRead};

/// The kinds' first and last records.
#[derive(Clone)]
pub struct KindRecords {
    /// Each kind's first record.
    pub first_records: [u32; 256],
    /// Each kind's last record.
    pub last_records: [u32; 259],
}

impl KindRecords {
    pub const fn new() -> Self {
        Self {
            first_records: [0; 256],
            last_records: [0; 259],
        }
    }
}

/// The records as one reader finds them: the game's thread's own (read through the store when
/// used), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum KindRecordsRef<'a> {
    Store(&'a DrawPlain<KindRecords>),
    Copy(&'a KindRecords),
}

impl KindRecordsRef<'_> {
    /// Kind `kind`'s first and last record (a first of -1: none).
    pub fn first_and_last(self, kind: usize) -> (u32, u32) {
        match self {
            Self::Store(store) => {
                let records = store.borrow();
                (records.first_records[kind], records.last_records[kind])
            }
            Self::Copy(records) => (records.first_records[kind], records.last_records[kind]),
        }
    }
}

impl<'a> From<&'a PlainRead<KindRecords>> for KindRecordsRef<'a> {
    fn from(copy: &'a PlainRead<KindRecords>) -> Self {
        Self::Copy(copy)
    }
}
