//! Plain records: the game's state as plain values (`u32`, `f32`, arrays, records of them), held
//! by one owner and copied with `Clone`. `PlainRecord` adds what the stores of cells gave the
//! game beyond that: the field-wise merge of what one copy changed into another (a view's own
//! copy of the cars), and the walk of every field by name that the simulation hash and the
//! tests read. The walk reports the same paths and bytes as a record of cells of the same
//! shape (`Cells::visit`), so a store moved from cells to plain values keeps its hash.
//!
//! `#[derive(rc2k_derive::PlainRecord)]` writes it for a struct, field by field. A field that
//! was a run of byte cells (`Bytes<N>`), reported as one run with no index, is marked
//! `#[plain(bytes)]`.

use crate::memory::cells::CellVisitor;

/// A plain record: copied with `Clone`, merged and walked field by field.
#[cfg_attr(
    not(any(test, feature = "remote")),
    allow(
        dead_code,
        reason = "walked only by the simulation hash and the tests so far"
    )
)]
pub trait PlainRecord: Clone {
    /// The fields of `now` that differ from the same fields of `start` set into this record,
    /// the others left as they are: what one copy changed since it was `start`, put into
    /// another. Floats compare by their bits.
    #[allow(
        dead_code,
        reason = "the merge is for the stores still on cells (view copies)"
    )]
    fn merge_changes(&mut self, now: &Self, start: &Self);

    /// Every primitive run of the record handed to `visitor`, with the field names and array
    /// indices on the way, as `Cells::visit` reports a record of cells of the same shape.
    fn visit(&self, visitor: &mut dyn CellVisitor);

    /// A whole slice of this type reported as one run, when it is a primitive (an array of
    /// them is one run, element 0): `index(0)`, the leaf, `leave`. Says whether it did.
    fn visit_slice(_slice: &[Self], _visitor: &mut dyn CellVisitor) -> bool {
        false
    }
}

/// A primitive: one value whose bytes are its little-endian ones.
macro_rules! primitive {
    ($($t:ty => $same:expr),* $(,)?) => {$(
        impl PlainRecord for $t {
            fn merge_changes(&mut self, now: &Self, start: &Self) {
                let same: fn(&$t, &$t) -> bool = $same;
                if !same(now, start) {
                    *self = *now;
                }
            }

            fn visit(&self, visitor: &mut dyn CellVisitor) {
                visitor.leaf(&self.to_le_bytes());
            }

            fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
                visitor.index(0);
                if !slice.is_empty() {
                    let bytes: Vec<u8> = slice.iter().flat_map(|v| v.to_le_bytes()).collect();
                    visitor.leaf(&bytes);
                }
                visitor.leave();
                true
            }
        }
    )*};
}

primitive! {
    u8 => |a, b| a == b,
    u16 => |a, b| a == b,
    u32 => |a, b| a == b,
    i32 => |a, b| a == b,
    f32 => |a, b| a.to_bits() == b.to_bits(),
}

impl<T: PlainRecord, const N: usize> PlainRecord for [T; N] {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        for ((to, now), start) in self.iter_mut().zip(now).zip(start) {
            to.merge_changes(now, start);
        }
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        if N == 0 || T::visit_slice(self.as_slice(), visitor) {
            return;
        }
        for (i, element) in self.iter().enumerate() {
            visitor.index(i);
            element.visit(visitor);
            visitor.leave();
        }
    }
}

/// A run of bytes reported as one run with no index, as a field of byte cells (`Bytes<N>`) is:
/// what `#[plain(bytes)]` fields are walked with.
#[cfg_attr(
    not(any(test, feature = "remote")),
    allow(
        dead_code,
        reason = "walked only by the simulation hash and the tests so far"
    )
)]
pub fn visit_bytes(bytes: &[u8], visitor: &mut dyn CellVisitor) {
    if !bytes.is_empty() {
        visitor.leaf(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::cells::Cells;
    use crate::memory::plain_floats::PlainFloats;
    use crate::memory::session::{Bytes, PlainDword, PlainFloat};

    /// Every call a walk makes, as text.
    #[derive(Default)]
    struct Trace(Vec<String>);

    impl CellVisitor for Trace {
        fn enter(&mut self, name: &'static str) {
            self.0.push(format!("enter {name}"));
        }
        fn index(&mut self, i: usize) {
            self.0.push(format!("index {i}"));
        }
        fn leave(&mut self) {
            self.0.push("leave".into());
        }
        fn leaf(&mut self, bytes: &[u8]) {
            self.0.push(format!("leaf {bytes:?}"));
        }
    }

    #[derive(rc2k_derive::Cells)]
    struct InCells {
        speed: PlainFloat,
        row: PlainFloats<3>,
        count: PlainDword,
        words: [PlainDword; 2],
        switches: Bytes<3>,
        pairs: [[PlainDword; 2]; 2],
    }

    #[derive(Clone, rc2k_derive::PlainRecord)]
    struct Plain {
        speed: f32,
        row: [f32; 3],
        count: u32,
        words: [u32; 2],
        #[plain(bytes)]
        switches: [u8; 3],
        pairs: [[u32; 2]; 2],
    }

    fn walk_cells(r: &InCells) -> Vec<String> {
        let mut t = Trace::default();
        r.visit(&mut t);
        t.0
    }

    fn walk_plain(r: &Plain) -> Vec<String> {
        let mut t = Trace::default();
        r.visit(&mut t);
        t.0
    }

    #[test]
    fn a_plain_record_walks_as_the_cells_of_its_shape() {
        let cells = InCells {
            speed: PlainFloat::new(1.5),
            row: PlainFloats::from_bits([1.0f32.to_bits(), 2.0f32.to_bits(), 0x7FC0_0001]),
            count: PlainDword::new(7),
            words: [PlainDword::new(8), PlainDword::new(9)],
            switches: Bytes::new([1, 2, 3]),
            pairs: [
                [PlainDword::new(1), PlainDword::new(2)],
                [PlainDword::new(3), PlainDword::new(4)],
            ],
        };
        let plain = Plain {
            speed: 1.5,
            row: [1.0, 2.0, f32::from_bits(0x7FC0_0001)],
            count: 7,
            words: [8, 9],
            switches: [1, 2, 3],
            pairs: [[1, 2], [3, 4]],
        };
        assert_eq!(walk_plain(&plain), walk_cells(&cells));
    }

    #[test]
    fn a_merge_takes_the_changed_fields_only_and_floats_by_bits() {
        let start = Plain {
            speed: 0.0,
            row: [1.0, 2.0, 3.0],
            count: 1,
            words: [1, 2],
            switches: [0; 3],
            pairs: [[0; 2]; 2],
        };
        let mut now = start.clone();
        now.speed = -0.0;
        now.row[1] = 5.0;
        now.words[0] = 9;
        now.switches[2] = 4;
        let mut to = Plain {
            speed: 3.0,
            row: [7.0; 3],
            count: 2,
            words: [3, 4],
            switches: [6; 3],
            pairs: [[5; 2]; 2],
        };
        to.merge_changes(&now, &start);
        assert_eq!(to.speed.to_bits(), (-0.0f32).to_bits());
        assert_eq!(to.row, [7.0, 5.0, 7.0]);
        assert_eq!((to.count, to.words), (2, [9, 4]));
        assert_eq!(to.switches, [6, 6, 4]);
        assert_eq!(to.pairs, [[5; 2]; 2]);
    }
}
