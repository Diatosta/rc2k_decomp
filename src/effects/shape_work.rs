//! The shape sets' work.

use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// A shape set (IDA `MarkShapes`, 0xAB8 bytes): the working block `g_marksFarthestA` .. and each set copied from it.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarkShapes {
    /// 0x0: `g_marksFarthestA`, `dword_884FB8`: list A's farthest pair (inner, outer)
    pub farthest_a: [PlainDword; 2],
    /// 0x8: `g_marksFarthestB`, `dword_884FC0`: list B's farthest pair
    pub farthest_b: [PlainDword; 2],
    /// 0x10: `dword_884FC4` .. `dword_884FE0`: a, b, c, d scaled, two zeros, the two tilts
    pub steps: [PlainDword; 8],
    /// 0x38: `g_marksPointsACount`
    pub count_a: PlainDword,
    /// 0x3C: `g_marksPointsBCount`
    pub count_b: PlainDword,
    /// 0x40: `g_marksPointsA`: list A's point ids (then pointers into the object's points)
    pub ids_a: [PlainDword; 64],
    /// 0x140: `g_marksPointsB`
    pub ids_b: [PlainDword; 64],
    /// 0x240: `g_marksPointsAFaces`: the face of each point of list A
    pub faces_a: [PlainDword; 64],
    /// 0x340: `g_marksPointsBFaces`
    pub faces_b: [PlainDword; 64],
    /// 0x440: `g_marksTurnedA`: 64 points of 12 bytes, turned
    pub turned_a: PlainBytes<768>,
    /// 0x740: `g_marksTurnedB`
    pub turned_b: PlainBytes<768>,
    /// 0xA40: `dword_8859F4` ..: list A's lowest point
    pub low_a: [PlainDword; 3],
    /// 0xA4C: `dword_885A00` ..: list B's lowest point
    pub low_b: [PlainDword; 3],
    /// 0xA58: list A's 3 x 3 turn (X times Z)
    pub matrix_a: [PlainFloat; 9],
    /// 0xA88: list B's 3 x 3 turn
    pub matrix_b: [PlainFloat; 9],
}

/// The shape sets' work (IDA 0x884E5C .. 0x89BCA4).
impl MarkShapes {
    /// `other` copied over this one, field by field (`memory::move_bytes` of the 0xAB8
    /// bytes).
    pub fn copy_from(&self, other: &MarkShapes) {
        // Bit-exact copy of 32-bit cells, whatever float or whole number they hold.
        fn cells<C: crate::session::Cell32>(to: &[C], from: &[C]) {
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

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ShapeWork {
    /// 0x884E5C: `g_marksShapeFirstPoint`
    pub first_point: PlainDword,
    /// 0x884E60: `g_marksShapeTurnX`: the turn about X (3 x 3)
    pub turn_x: [PlainFloat; 9],
    /// 0x884F50: `g_marksShapeTurnA`: the turn about Z by list A's angle
    pub turn_a: [PlainFloat; 9],
    /// 0x884F80: `g_marksShapeTurnB`
    pub turn_b: [PlainFloat; 9],
    /// 0x884FB0: `g_marksShapeSets`: how many sets are made
    pub sets_made: PlainDword,
    /// 0x884FB4: the working block a set is made in (`g_marksFarthestA` ..)
    pub work: MarkShapes,
    /// 0x885A6C: `g_marksShapeSet`: the made sets (the game makes at most 19; the nominal 0x22nd would run past 0x89C000)
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
