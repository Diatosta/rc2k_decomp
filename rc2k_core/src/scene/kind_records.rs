//! The object kinds' records: 1200 records of 0x80 bytes, filled from four files.

use crate::Invalid;

/// Bytes of one kind's record.
pub const RECORD_LEN: usize = 0x80;
/// How many kinds there are.
pub const KINDS: usize = 1200;

/// The kinds one file holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindRange {
    pub first_kind: usize,
    pub kinds: usize,
}

/// The four files' ranges in the order they are read: the objects', the front end's, the cars'
/// and the extras'.
pub const RANGES: [KindRange; 4] = [
    KindRange {
        first_kind: 0,
        kinds: 0x190,
    },
    KindRange {
        first_kind: 0x258,
        kinds: 0xC8,
    },
    KindRange {
        first_kind: 0x190,
        kinds: 0xC8,
    },
    KindRange {
        first_kind: 0x320,
        kinds: 0x190,
    },
];

/// The records: `KINDS` x `RECORD_LEN` bytes, files laid over what was held.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindRecords {
    bytes: Vec<u8>,
}

impl KindRecords {
    /// The records held now (`KINDS * RECORD_LEN` bytes).
    pub fn from_held(held: &[u8]) -> Result<Self, Invalid> {
        if held.len() != KINDS * RECORD_LEN {
            return Err(Invalid("kind records of the wrong size"));
        }
        Ok(Self {
            bytes: held.to_vec(),
        })
    }

    /// A file's first bytes laid over `range` (all of them, or the range's room); what the file
    /// does not reach is kept.
    pub fn overlay(&mut self, range: KindRange, file: &[u8]) {
        let start = range.first_kind * RECORD_LEN;
        let room = (range.kinds * RECORD_LEN).min(self.bytes.len().saturating_sub(start));
        let n = file.len().min(room);
        self.bytes[start..start + n].copy_from_slice(&file[..n]);
    }

    /// The records' bytes, by kind.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_lie_inside_the_records_and_apart() {
        let mut spans: Vec<_> = RANGES
            .iter()
            .map(|r| (r.first_kind, r.first_kind + r.kinds))
            .collect();
        spans.sort_unstable();
        assert!(spans.iter().all(|&(_, end)| end <= KINDS));
        assert!(spans.windows(2).all(|w| w[0].1 <= w[1].0));
    }

    #[test]
    fn a_short_file_leaves_the_rest_and_a_long_one_stops_at_its_range() {
        let mut records = KindRecords::from_held(&vec![0xAA; KINDS * RECORD_LEN]).unwrap();
        let cars = RANGES[2];
        records.overlay(cars, &[1, 2, 3]);
        let at = cars.first_kind * RECORD_LEN;
        assert_eq!(&records.bytes()[at..at + 4], &[1, 2, 3, 0xAA]);
        records.overlay(cars, &vec![7; cars.kinds * RECORD_LEN + 50]);
        let end = at + cars.kinds * RECORD_LEN;
        assert_eq!(records.bytes()[end - 1], 7);
        assert_eq!(records.bytes()[end], 0xAA);
        assert_eq!(records.bytes()[at - 1], 0xAA);
    }

    #[test]
    fn records_of_the_wrong_size_are_refused() {
        assert!(KindRecords::from_held(&[0; 10]).is_err());
    }
}
