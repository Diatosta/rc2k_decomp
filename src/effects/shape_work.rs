//! The shape sets' work out of the marks' block (phase 7).

use crate::session::{Bytes, Dword, Float};

/// A shape set (IDA `MarkShapes`, 0xAB8 bytes): the working block `g_marksFarthestA` .. and each set copied from it.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarkShapes {
    /// 0x0: `g_marksFarthestA`, `dword_884FB8`: list A's farthest pair (inner, outer)
    pub farthest_a: [Dword; 2],
    /// 0x8: `g_marksFarthestB`, `dword_884FC0`: list B's farthest pair
    pub farthest_b: [Dword; 2],
    /// 0x10: `dword_884FC4` .. `dword_884FE0`: a, b, c, d scaled, two zeros, the two tilts
    pub steps: [Dword; 8],
    /// 0x30: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_30: Bytes<8>,
    /// 0x38: `g_marksPointsACount`
    pub count_a: Dword,
    /// 0x3C: `g_marksPointsBCount`
    pub count_b: Dword,
    /// 0x40: `g_marksPointsA`: list A's point ids (then pointers into the object's points)
    pub ids_a: [Dword; 64],
    /// 0x140: `g_marksPointsB`
    pub ids_b: [Dword; 64],
    /// 0x240: `g_marksPointsAFaces`: the face of each point of list A
    pub faces_a: [Dword; 64],
    /// 0x340: `g_marksPointsBFaces`
    pub faces_b: [Dword; 64],
    /// 0x440: `g_marksTurnedA`: 64 points of 12 bytes, turned
    pub turned_a: Bytes<768>,
    /// 0x740: `g_marksTurnedB`
    pub turned_b: Bytes<768>,
    /// 0xA40: `dword_8859F4` ..: list A's lowest point
    pub low_a: [Dword; 3],
    /// 0xA4C: `dword_885A00` ..: list B's lowest point
    pub low_b: [Dword; 3],
    /// 0xA58: list A's 3 x 3 turn (X times Z)
    pub matrix_a: [Float; 9],
    /// 0xA7C: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_a7c: Bytes<12>,
    /// 0xA88: list B's 3 x 3 turn
    pub matrix_b: [Float; 9],
    /// 0xAAC: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_aac: Bytes<12>,
}

/// The shape sets' work (IDA 0x884E5C .. 0x89BCA4, the image's layout; phase 7, typed state).
impl MarkShapes {
    /// `other` copied over this one, field by field (`memory::move_bytes` of the 0xAB8
    /// bytes).
    pub fn copy_from(&self, other: &MarkShapes) {
        fn cells<C: crate::session::Cell32>(to: &[C], from: &[C]) {
            for (t, f) in to.iter().zip(from) {
                t.set_bits(f.bits());
            }
        }
        cells(&self.farthest_a, &other.farthest_a);
        cells(&self.farthest_b, &other.farthest_b);
        cells(&self.steps, &other.steps);
        self._gap_30.write(&other._gap_30.read());
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
        self._gap_a7c.write(&other._gap_a7c.read());
        cells(&self.matrix_b, &other.matrix_b);
        self._gap_aac.write(&other._gap_aac.read());
    }
}

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ShapeWork {
    /// 0x884E5C: `g_marksShapeFirstPoint`
    pub first_point: Dword,
    /// 0x884E60: `g_marksShapeTurnX`: the turn about X (3 x 3)
    pub turn_x: [Float; 9],
    /// 0x884E84: Not named: 204 bytes (a gap, or unreferenced data).
    pub _gap_884e84: Bytes<204>,
    /// 0x884F50: `g_marksShapeTurnA`: the turn about Z by list A's angle
    pub turn_a: [Float; 9],
    /// 0x884F74: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_884f74: Bytes<12>,
    /// 0x884F80: `g_marksShapeTurnB`
    pub turn_b: [Float; 9],
    /// 0x884FA4: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_884fa4: Bytes<12>,
    /// 0x884FB0: `g_marksShapeSets`: how many sets are made
    pub sets_made: Dword,
    /// 0x884FB4: the working block a set is made in (`g_marksFarthestA` ..)
    pub work: MarkShapes,
    /// 0x885A6C: `g_marksShapeSet`: the made sets (the game makes at most 19; the nominal 0x22nd would run past 0x89C000)
    pub sets: [MarkShapes; 33],
    /// 0x89BC24: Not named: 128 bytes (a gap, or unreferenced data).
    pub _gap_89bc24: Bytes<128>,
}

impl ShapeWork {
    pub const fn new() -> Self {
        ShapeWork {
            first_point: Dword::new(0x0),
            turn_x: [const { Float::from_bits(0x0) }; 9],
            _gap_884e84: Bytes::new([0; 204]),
            turn_a: [const { Float::from_bits(0x0) }; 9],
            _gap_884f74: Bytes::new([0; 12]),
            turn_b: [const { Float::from_bits(0x0) }; 9],
            _gap_884fa4: Bytes::new([0; 12]),
            sets_made: Dword::new(0x0),
            work: MarkShapes {
                farthest_a: [const { Dword::new(0x0) }; 2],
                farthest_b: [const { Dword::new(0x0) }; 2],
                steps: [const { Dword::new(0x0) }; 8],
                _gap_30: Bytes::new([0; 8]),
                count_a: Dword::new(0x0),
                count_b: Dword::new(0x0),
                ids_a: [const { Dword::new(0x0) }; 64],
                ids_b: [const { Dword::new(0x0) }; 64],
                faces_a: [const { Dword::new(0x0) }; 64],
                faces_b: [const { Dword::new(0x0) }; 64],
                turned_a: Bytes::new([0; 768]),
                turned_b: Bytes::new([0; 768]),
                low_a: [const { Dword::new(0x0) }; 3],
                low_b: [const { Dword::new(0x0) }; 3],
                matrix_a: [const { Float::from_bits(0x0) }; 9],
                _gap_a7c: Bytes::new([0; 12]),
                matrix_b: [const { Float::from_bits(0x0) }; 9],
                _gap_aac: Bytes::new([0; 12]),
            },
            sets: [const {
                MarkShapes {
                    farthest_a: [const { Dword::new(0x0) }; 2],
                    farthest_b: [const { Dword::new(0x0) }; 2],
                    steps: [const { Dword::new(0x0) }; 8],
                    _gap_30: Bytes::new([0; 8]),
                    count_a: Dword::new(0x0),
                    count_b: Dword::new(0x0),
                    ids_a: [const { Dword::new(0x0) }; 64],
                    ids_b: [const { Dword::new(0x0) }; 64],
                    faces_a: [const { Dword::new(0x0) }; 64],
                    faces_b: [const { Dword::new(0x0) }; 64],
                    turned_a: Bytes::new([0; 768]),
                    turned_b: Bytes::new([0; 768]),
                    low_a: [const { Dword::new(0x0) }; 3],
                    low_b: [const { Dword::new(0x0) }; 3],
                    matrix_a: [const { Float::from_bits(0x0) }; 9],
                    _gap_a7c: Bytes::new([0; 12]),
                    matrix_b: [const { Float::from_bits(0x0) }; 9],
                    _gap_aac: Bytes::new([0; 12]),
                }
            }; 33],
            _gap_89bc24: Bytes::new([0; 128]),
        }
    }
}

impl ShapeWork {
    pub const START: usize = 0x884E5C;
    pub const LEN: usize = 0x16E48;
}

#[cfg(test)]
mod tests_shape_work {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn shape_work_starts_as_the_games() {
        assert_eq!(size_of::<ShapeWork>(), ShapeWork::LEN);
        assert_eq!(
            offset_of!(ShapeWork, first_point),
            0x884E5C - ShapeWork::START
        );
        assert_eq!(offset_of!(ShapeWork, turn_x), 0x884E60 - ShapeWork::START);
        assert_eq!(offset_of!(ShapeWork, turn_a), 0x884F50 - ShapeWork::START);
        assert_eq!(offset_of!(ShapeWork, turn_b), 0x884F80 - ShapeWork::START);
        assert_eq!(
            offset_of!(ShapeWork, sets_made),
            0x884FB0 - ShapeWork::START
        );
        assert_eq!(offset_of!(ShapeWork, work), 0x884FB4 - ShapeWork::START);
        assert_eq!(offset_of!(ShapeWork, sets), 0x885A6C - ShapeWork::START);
        let p = core::ptr::addr_of!(crate::effects::owner::EFFECTS.windscreen_drops.shape_work)
            .cast::<u8>();
        let now: Vec<u8> = (0..ShapeWork::LEN)
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            now == crate::moved::image_bytes(ShapeWork::START, ShapeWork::LEN),
            "SHAPE_WORK starts differently from the image"
        );
    }
}
