//! The piece from the input keys word to the spin axis: the face-key factors, the course memory's
//! lists and pointers, the object and point-record pointers, the matrix work areas (squaring, turn,
//! inversion, angles), the two root tables and the spin variables. It stops before the car records
//! (`race::car_specs_store`). Plain cells: a drawing store (`session::DrawShared`), written on the
//! game's thread between frames and read by the drawing parts through their `workers::DrawReads`.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CourseLists {
    /// The mask of input keys read for the car's controls (`drive::read_car_controls`).
    pub input_keys_mask: PlainDword,
    /// Floats for the face sort key (`scene::face_sort_key`, `scene::fill_engine_tables`).
    pub face_key_factors: [PlainFloat; 256],
    /// `course_memory_list`, a MemoryList of 16 bytes (`race::level_init::allocate_course_memory`,
    /// `race::stage_free`).
    pub course_memory_list: crate::memory::ListCells,
    /// Unused; it only keeps the image's layout.
    pub unused_414: [PlainDword; 12],
    /// The second course MemoryList: set up by `race::level_init::allocate_course_memory` and
    /// freed by `race::stage_free`; nothing allocates from it.
    pub second_course_list: crate::memory::ListCells,
    /// Unused; it only keeps the image's layout.
    pub unused_454: [PlainDword; 12],
    /// A MemoryList (`scene::allocate_object_memory`).
    pub object_points_list: crate::memory::ListCells,
    /// Unused; it only keeps the image's layout.
    pub unused_494: [PlainDword; 28],
    /// A MemoryList (`scene::allocate_point_records`).
    pub point_records_list: crate::memory::ListCells,
    /// Unused; it only keeps the image's layout.
    pub unused_514: [PlainDword; 28],
    /// A memory list; unused.
    pub list_6ed0: [PlainDword; 4],
    /// Unused; it only keeps the image's layout.
    pub unused_594: [PlainDword; 28],
    /// The mud's MemoryList (`race::mud`).
    pub mud_list: crate::memory::ListCells,
    /// Unused; it only keeps the image's layout.
    pub unused_614: [PlainDword; 12],
    /// Bytes taken from the course lists, added and subtracted by the allocators
    /// (`race::stage_free`, `race::mud`, `scene::allocate_object_memory`).
    pub course_bytes_total: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_course_memory: PlainDword,
    /// Set when the second course list is freed (cleared by `race::stage_free`).
    pub second_course_list_a: PlainDword,
    /// As its neighbour (cleared by `race::stage_free`).
    pub second_course_list_b: PlainDword,
    /// Pointer to the last 0x400000 bytes of the course memory (`race::level_init`,
    /// `effects::profiles`, `effects::draw_frame`).
    pub course_info: PlainDword,
    /// Pointer to the object points (`scene::objects`, `scene::place_object_points`,
    /// `physics::object_hits_car`).
    pub object_points: PlainDword,
    /// Pointer (`scene::objects`, `race::stage_free`).
    pub face_memory: PlainDword,
    /// Bytes left for the object points (`scene::objects`).
    pub object_points_room: PlainDword,
    /// Bytes left for the faces (`scene::objects`).
    pub face_memory_room: PlainDword,
    /// Unused.
    pub dword_6fb4_unused: PlainDword,
    /// Pointer to the point records (`scene::project_point`, `race::frame_draw`,
    /// `race::car_shadow`).
    pub point_records: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_670: [PlainDword; 2],
    /// Not read or written by any code; it only keeps the image's layout.
    pub unused_678: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_first_part: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_680: PlainDword,
    /// Work of `scene::square_matrix`, row 0 (`scene::view_matrix`); now a local there; unused.
    pub squared_row_0: [PlainFloat; 4],
    /// Work of `scene::square_matrix`, row 1; now a local there; unused.
    pub squared_row_1: [PlainFloat; 4],
    /// Row 2 and the four dwords after, copied back (`scene::square_matrix`); now a local there;
    /// unused.
    pub squared_row_2_and_after: [PlainFloat; 8],
    /// The turn matrix `physics::move_slot` and `physics::spin_matrix` make with
    /// `physics::axis_turn_matrix`.
    pub turn_matrix: [PlainFloat; 16],
    /// The nine floats copied by `scene::invert_turn` (`scene::view_matrix`); no reader: unused.
    pub invert_copy: [PlainFloat; 9],
    /// The nine floats made by `scene::invert_turn`; no reader: unused.
    pub invert_made: [PlainFloat; 9],
    /// Sine of the angle (`physics::axis_turn_matrix`). The words after it are its work values.
    pub turn_sine: PlainFloat,
    /// Cosine (`physics::axis_turn_matrix`).
    pub turn_cosine: PlainFloat,
    /// 1 - cosine.
    pub turn_one_less_cosine: PlainFloat,
    /// (1 - cos) x z.
    pub turn_product_xz: PlainFloat,
    /// (1 - cos) x y.
    pub turn_product_xy: PlainFloat,
    /// (1 - cos) y z.
    pub turn_product_yz: PlainFloat,
    /// X times sine.
    pub turn_sine_x: PlainFloat,
    /// Y times sine.
    pub turn_sine_y: PlainFloat,
    /// Z times sine.
    pub turn_sine_z: PlainFloat,
    /// Divisor of the vector normalisation (the vector's length).
    pub vector_length: PlainFloat,
    /// The unit vector's x (the vector divided by `vector_length`), written by the normalisation.
    pub vector_unit_x: PlainFloat,
    /// The unit vector's y, as above.
    pub vector_unit_y: PlainFloat,
    /// The unit vector's z, as above.
    pub vector_unit_z: PlainFloat,
    /// Unused; it only keeps the image's layout.
    pub unused_780: PlainDword,
    /// A scratch float.
    pub vector_scratch: PlainFloat,
    /// The car's pitch: minus the clamped asin of the turned forward vector's y
    /// (`physics::matrix_angles`, `physics::car_angles`).
    pub car_pitch: PlainFloat,
    /// The car's heading: atan2 of the turned forward vector's x over z (`physics::matrix_angles`).
    pub car_heading: PlainFloat,
    /// The car's roll: asin of the turned side vector's y over the pitch's cosine
    /// (`physics::matrix_angles`).
    pub car_roll: PlainFloat,
    /// The turned unit vector's x (`physics::matrix_angles`).
    pub turned_x: PlainFloat,
    /// Its y.
    pub turned_y: PlainFloat,
    /// Its z.
    pub turned_z: PlainFloat,
    /// Unused; it only keeps the image's layout.
    pub unused_7a0: [PlainDword; 3],
    /// Square roots by mantissa bits (`math::normalise_vector`, `scene::fill_engine_tables`,
    /// `race::frame_draw`).
    pub root_table: [PlainFloat; 256],
    /// Roots by exponent (same users).
    pub exponent_root_table: [PlainFloat; 256],
    /// The angle of the turn (`physics::axis_turn_matrix`, `physics::spin_matrix`,
    /// `physics::move_slot`).
    pub spin_angle: PlainFloat,
    /// The axis' x, y, z.
    pub spin_axis: [PlainFloat; 3],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
impl CourseLists {
    pub const fn new() -> Self {
        Self {
            unused_7a0: [
                PlainDword::new(0x4C22F983),
                PlainDword::new(0x4422F983),
                PlainDword::new(0x30490FDB),
            ],
            ..ZeroCells::ZERO
        }
    }
}
