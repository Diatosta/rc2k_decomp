//! The shape sets' inputs and tuning.

/// The shape sets' inputs (`effects::shape_points`, `effects::turns`).
pub struct ShapeInputs {
    /// The tilts a shape set is made with (`make_shape_set`: copied into the working block;
    /// `make_shape_sets` writes both).
    pub tilts: [u32; 2],
    /// The set's four ends (`make_shape_sets` writes them, `make_shape_set` scales them into the
    /// working block).
    pub ends: [f32; 4],
}

impl ShapeInputs {
    pub const fn new() -> Self {
        ShapeInputs {
            tilts: [0x0, 0x0],
            ends: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

/// The shape sets' tables and constants (`effects::make_shape_sets`, `make_shape_set`,
/// `turn_all_shapes`).
pub struct ShapeSetTuning {
    /// The two face kinds the shape sets are made for (set by `effects::make_shape_sets` from the
    /// kind's entries of the two tables that follow).
    pub face_kinds: [u32; 2],
    /// The kind's first face kind, by the marks' kind (0x22 entries).
    pub first_face_kind: [u32; 34],
    /// The kind's second face kind, by the marks' kind (0x22 entries).
    pub second_face_kind: [u32; 34],
    /// The second shape's end by the marks' kind (floats: -80, -90 .. as bits; 0x22 entries), put
    /// into the set's inputs.
    pub end_b: [f32; 34],
    /// The first shape's end by the marks' kind (floats as bits; 0x22 entries), put into the set's
    /// inputs.
    pub end_a: [f32; 34],
    /// The sets' second ends of the first and second shape (0 at first).
    pub set_ends: [f32; 2],
    /// Each shape's eased turn from its first angle (0) and a second (-pi / 2; the second
    /// is not read), as bits (`effects::turn_all_shapes`).
    pub turn_angles: [f32; 4],
    /// 0.0175 (degrees to radians) four times, the set's step scales (`effects::make_shape_set`).
    pub step_scales: [f32; 4],
}

impl ShapeSetTuning {
    pub const fn new() -> Self {
        ShapeSetTuning {
            face_kinds: [0x1E, 0x1D],
            first_face_kind: [
                0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E,
                0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF, 0x1E,
            ],
            second_face_kind: [
                0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D,
                0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x10, 0x1D, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF, 0x1D,
            ],
            end_b: [
                -80.0, -80.0, -80.0, -80.0, -80.0, -80.0, -80.0, -80.0, -80.0, -80.0, 90.0, -80.0,
                90.0, -80.0, -80.0, -80.0, -80.0, -80.0, -80.0, -80.0, -80.0, -80.0, 75.0, -90.0,
                -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -80.0,
            ],
            end_a: [
                -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, 80.0, -90.0,
                80.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -90.0, -75.0, 99.0, 90.0,
                90.0, 90.0, 90.0, 90.0, 90.0, 90.0, 90.0, 90.0, -90.0, -90.0,
            ],
            set_ends: [0.0, 0.0],
            turn_angles: [0.0, -1.5707964, 0.0, -1.5707964],
            step_scales: [0.017453292, 0.017453292, 0.017453292, 0.017453292],
        }
    }
}
