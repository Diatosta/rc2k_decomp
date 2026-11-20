//! The shape sets' inputs and tuning.

use crate::memory::session::{PlainDword, PlainFloat};

/// The shape sets' inputs (`effects::shape_points`, `effects::turns`).
#[derive(rc2k_derive::Cells)]
pub struct ShapeInputs {
    /// The angle in hand while a turn matrix is made (`turn_shape_points`, `smooth_steps`; a
    /// scratch, nothing reads it back).
    pub scratch: PlainDword,
    /// The tilts a shape set is made with (`make_shape_set`: copied into the working block;
    /// `make_shape_sets` writes both).
    pub tilts: [PlainDword; 2],
    /// The set's four ends (`make_shape_sets` writes them, `make_shape_set` scales them into the
    /// working block).
    pub ends: [PlainFloat; 4],
}

impl ShapeInputs {
    pub const fn new() -> Self {
        ShapeInputs {
            scratch: PlainDword::new(0x0),
            tilts: [PlainDword::new(0x0), PlainDword::new(0x0)],
            ends: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
        }
    }
}

/// The shape sets' tables and constants (`effects::make_shape_sets`, `make_shape_set`,
/// `turn_all_shapes`).
#[derive(rc2k_derive::Cells)]
pub struct ShapeSetTuning {
    /// The two face kinds the shape sets are made for (set by `effects::make_shape_sets` from the
    /// kind's entries of the two tables that follow).
    pub face_kinds: [PlainDword; 2],
    /// The kind's first face kind, by the marks' kind (0x22 entries).
    pub first_face_kind: [PlainDword; 34],
    /// The kind's second face kind, by the marks' kind (0x22 entries).
    pub second_face_kind: [PlainDword; 34],
    /// The second shape's end by the marks' kind (floats: -80, -90 .. as bits; 0x22 entries), put
    /// into the set's inputs.
    pub end_b: [PlainFloat; 34],
    /// The first shape's end by the marks' kind (floats as bits; 0x22 entries), put into the set's
    /// inputs.
    pub end_a: [PlainFloat; 34],
    /// Four (end, 0) pairs, -90, -80, 80, 90 (nothing reads them).
    pub end_pairs: [PlainFloat; 8],
    /// The sets' second ends of the first and second shape (0 at first).
    pub set_ends: [PlainFloat; 2],
    /// Each shape's eased turn from its first angle (0) and a second (-pi / 2; the second
    /// is not read), as bits (`effects::turn_all_shapes`).
    pub turn_angles: [PlainFloat; 4],
    /// 0.0175 (degrees to radians) four times, the set's step scales (`effects::make_shape_set`).
    pub step_scales: [PlainFloat; 4],
}

impl ShapeSetTuning {
    pub const fn new() -> Self {
        ShapeSetTuning {
            face_kinds: [PlainDword::new(0x1E), PlainDword::new(0x1D)],
            first_face_kind: [
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0x1E),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1E),
            ],
            second_face_kind: [
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x1D),
                PlainDword::new(0x10),
                PlainDword::new(0x1D),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1D),
            ],
            end_b: [
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(90.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(90.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(75.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-80.0),
            ],
            end_a: [
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(80.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(80.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-75.0),
                PlainFloat::new(99.0),
                PlainFloat::new(90.0),
                PlainFloat::new(90.0),
                PlainFloat::new(90.0),
                PlainFloat::new(90.0),
                PlainFloat::new(90.0),
                PlainFloat::new(90.0),
                PlainFloat::new(90.0),
                PlainFloat::new(90.0),
                PlainFloat::new(90.0),
                PlainFloat::new(-90.0),
                PlainFloat::new(-90.0),
            ],
            end_pairs: [
                PlainFloat::new(-90.0),
                PlainFloat::new(0.0),
                PlainFloat::new(-80.0),
                PlainFloat::new(0.0),
                PlainFloat::new(80.0),
                PlainFloat::new(0.0),
                PlainFloat::new(90.0),
                PlainFloat::new(0.0),
            ],
            set_ends: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            turn_angles: [
                PlainFloat::new(0.0),
                PlainFloat::new(-1.5707964),
                PlainFloat::new(0.0),
                PlainFloat::new(-1.5707964),
            ],
            step_scales: [
                PlainFloat::new(0.017453292),
                PlainFloat::new(0.017453292),
                PlainFloat::new(0.017453292),
                PlainFloat::new(0.017453292),
            ],
        }
    }
}
