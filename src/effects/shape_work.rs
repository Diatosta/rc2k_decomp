//! The shape sets' work.

use crate::memory::session::{PlainBytes, PlainDword, PlainFloat};

/// A shape set (0xAB8 bytes): the working block and each set copied from it.
#[derive(rc2k_derive::Cells)]
pub struct MarkShapes {
    /// +0x0: list A's farthest pair (inner, outer)
    pub farthest_a: [PlainDword; 2],
    /// +0x8: list B's farthest pair
    pub farthest_b: [PlainDword; 2],
    /// +0x10: a, b, c, d scaled, two zeros, the two tilts
    pub steps: [PlainDword; 8],
    /// +0x38: list A's point count
    pub count_a: PlainDword,
    /// +0x3C: list B's point count
    pub count_b: PlainDword,
    /// +0x40: list A's point ids (then pointers into the object's points)
    pub ids_a: [PlainDword; 64],
    /// +0x140: list B's point ids
    pub ids_b: [PlainDword; 64],
    /// +0x240: the face of each point of list A
    pub faces_a: [PlainDword; 64],
    /// +0x340: the face of each point of list B
    pub faces_b: [PlainDword; 64],
    /// +0x440: 64 points of 12 bytes, turned
    pub turned_a: PlainBytes<768>,
    /// +0x740: list B's turned points
    pub turned_b: PlainBytes<768>,
    /// +0xA40: list A's lowest point
    pub low_a: [PlainDword; 3],
    /// +0xA4C: list B's lowest point
    pub low_b: [PlainDword; 3],
    /// 0xA58: list A's 3 x 3 turn (X times Z)
    pub matrix_a: [PlainFloat; 9],
    /// 0xA88: list B's 3 x 3 turn
    pub matrix_b: [PlainFloat; 9],
}

/// The shape sets' work.
impl MarkShapes {
    /// `other` copied over this one, field by field (a copy of the 0xAB8
    /// bytes).
    pub fn copy_from(&self, other: &MarkShapes) {
        // Bit-exact copy of 32-bit cells, whatever float or whole number they hold.
        fn cells<C: crate::memory::session::Cell32>(to: &[C], from: &[C]) {
            for (t, f) in to.iter().zip(from) {
                t.set_bits(f.bits());
            }
        }
        cells(&self.farthest_a, &other.farthest_a);
        cells(&self.farthest_b, &other.farthest_b);
        cells(&self.steps, &other.steps);
        cells(
            core::slice::from_ref(&self.count_a),
            core::slice::from_ref(&other.count_a),
        );
        cells(
            core::slice::from_ref(&self.count_b),
            core::slice::from_ref(&other.count_b),
        );
        cells(&self.ids_a, &other.ids_a);
        cells(&self.ids_b, &other.ids_b);
        cells(&self.faces_a, &other.faces_a);
        cells(&self.faces_b, &other.faces_b);
        self.turned_a.write(&other.turned_a.read());
        self.turned_b.write(&other.turned_b.read());
        cells(&self.low_a, &other.low_a);
        cells(&self.low_b, &other.low_b);
        cells(&self.matrix_a, &other.matrix_a);
        cells(&self.matrix_b, &other.matrix_b);
    }
}

#[derive(rc2k_derive::Cells)]
pub struct ShapeWork {
    /// The first point of the shape being made
    pub first_point: PlainDword,
    /// The turn about X (3 x 3)
    pub turn_x: [PlainFloat; 9],
    /// The turn about Z by list A's angle
    pub turn_a: [PlainFloat; 9],
    /// The turn about Z by list B's angle
    pub turn_b: [PlainFloat; 9],
    /// How many sets are made
    pub sets_made: PlainDword,
    /// The working block a set is made in
    pub work: MarkShapes,
    /// The made sets (the game makes at most 19; the nominal 0x22nd would run past the range)
    pub sets: [MarkShapes; 33],
}

impl ShapeWork {
    pub const fn new() -> Self {
        ShapeWork {
            first_point: PlainDword::new(0x0),
            turn_x: [const { PlainFloat::new(0.0) }; 9],
            turn_a: [const { PlainFloat::new(0.0) }; 9],
            turn_b: [const { PlainFloat::new(0.0) }; 9],
            sets_made: PlainDword::new(0x0),
            work: MarkShapes {
                farthest_a: [const { PlainDword::new(0x0) }; 2],
                farthest_b: [const { PlainDword::new(0x0) }; 2],
                steps: [const { PlainDword::new(0x0) }; 8],
                count_a: PlainDword::new(0x0),
                count_b: PlainDword::new(0x0),
                ids_a: [const { PlainDword::new(0x0) }; 64],
                ids_b: [const { PlainDword::new(0x0) }; 64],
                faces_a: [const { PlainDword::new(0x0) }; 64],
                faces_b: [const { PlainDword::new(0x0) }; 64],
                turned_a: PlainBytes::new([0; 768]),
                turned_b: PlainBytes::new([0; 768]),
                low_a: [const { PlainDword::new(0x0) }; 3],
                low_b: [const { PlainDword::new(0x0) }; 3],
                matrix_a: [const { PlainFloat::new(0.0) }; 9],
                matrix_b: [const { PlainFloat::new(0.0) }; 9],
            },
            sets: [const {
                MarkShapes {
                    farthest_a: [const { PlainDword::new(0x0) }; 2],
                    farthest_b: [const { PlainDword::new(0x0) }; 2],
                    steps: [const { PlainDword::new(0x0) }; 8],
                    count_a: PlainDword::new(0x0),
                    count_b: PlainDword::new(0x0),
                    ids_a: [const { PlainDword::new(0x0) }; 64],
                    ids_b: [const { PlainDword::new(0x0) }; 64],
                    faces_a: [const { PlainDword::new(0x0) }; 64],
                    faces_b: [const { PlainDword::new(0x0) }; 64],
                    turned_a: PlainBytes::new([0; 768]),
                    turned_b: PlainBytes::new([0; 768]),
                    low_a: [const { PlainDword::new(0x0) }; 3],
                    low_b: [const { PlainDword::new(0x0) }; 3],
                    matrix_a: [const { PlainFloat::new(0.0) }; 9],
                    matrix_b: [const { PlainFloat::new(0.0) }; 9],
                }
            }; 33],
        }
    }
}
