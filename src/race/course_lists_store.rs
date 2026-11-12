//! The piece from the input keys word 0x5C694C to the spin axis at 0x5C7904: the face-key factors, the course memory's lists and pointers, the object and point-record pointers, the matrix work areas (squaring, turn, inversion, angles), the two root tables and the spin variables.
//! It stops before the car records at 0x5C790C (`race::car_specs_store`).
//! Plain cells: a drawing store (`session::DrawShared`), written on the game's thread between
//! frames and read by the drawing parts through their `workers::DrawReads`. Names are readings
//! unless IDA's.

use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CourseLists {
    /// +0x0 (0x5C694C): the mask of input keys read for the car's controls (`drive::read_car_controls`, `dword_5C694C`).
    pub input_keys_mask: PlainDword,
    /// +0x4 (0x5C6950): `g_faceKeyFactors`: floats for the face sort key (`scene::face_sort_key`, `scene::fill_engine_tables`).
    pub face_key_factors: [PlainFloat; 256],
    /// +0x404 (0x5C6D50): `g_courseMemoryList`, a MemoryList of 16 bytes (`race::allocate_course_memory`, `race::stage_free`).
    pub course_memory_list: [PlainDword; 4],
    /// +0x414 (0x5C6D60): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_414: [PlainDword; 12],
    /// +0x444 (0x5C6D90): `stru_5C6D90`, a second course MemoryList: set up by `race::allocate_course_memory` (0x41F8FE) and freed by `race::free_course_list_2`; nothing allocates from it (IDA xrefs: only those two).
    pub second_course_list: [PlainDword; 4],
    /// +0x454 (0x5C6DA0): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_454: [PlainDword; 12],
    /// +0x484 (0x5C6DD0): `stru_5C6DD0`, a MemoryList (`scene::allocate_object_memory`).
    pub object_points_list: [PlainDword; 4],
    /// +0x494 (0x5C6DE0): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_494: [PlainDword; 28],
    /// +0x504 (0x5C6E50): `stru_5C6E50`, a MemoryList (`scene::allocate_point_records`).
    pub point_records_list: [PlainDword; 4],
    /// +0x514 (0x5C6E60): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_514: [PlainDword; 28],
    /// +0x584 (0x5C6ED0): `stru_5C6ED0`, a MemoryList used by 0x459764, 0x4599D4 (not used by Rust).
    pub list_6ed0: [PlainDword; 4],
    /// +0x594 (0x5C6EE0): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_594: [PlainDword; 28],
    /// +0x604 (0x5C6F50): `stru_5C6F50`, the mud's MemoryList (`race::mud`).
    pub mud_list: [PlainDword; 4],
    /// +0x614 (0x5C6F60): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_614: [PlainDword; 12],
    /// +0x644 (0x5C6F90): `dword_5C6F90`: bytes taken from the course lists, added and subtracted by the allocators (`race::stage_free`, `race::mud`, `scene::allocate_object_memory`).
    pub course_bytes_total: PlainDword,
    /// +0x648 (0x5C6F94): `g_courseMemory`: handle of the course's memory, `level_init::COURSE_TAG` while taken (`race::course_data`, `race::norms`, `race::pace_notes`).
    pub course_memory: PlainDword,
    /// +0x64c (0x5C6F98): `dword_5C6F98`, set by `race::free_course_list_2` (cleared by `race::stage_free`).
    pub second_course_list_a: PlainDword,
    /// +0x650 (0x5C6F9C): `dword_5C6F9C`, as its neighbour (cleared by `race::stage_free`).
    pub second_course_list_b: PlainDword,
    /// +0x654 (0x5C6FA0): `g_courseInfo`: pointer to the last 0x400000 bytes of the course memory (`race::level_init`, `effects::profiles`, `effects::draw_frame`).
    pub course_info: PlainDword,
    /// +0x658 (0x5C6FA4): `g_objectPoints`: pointer to the object points (`scene::objects`, `scene::place_object_points`, `physics::object_hits_car`).
    pub object_points: PlainDword,
    /// +0x65c (0x5C6FA8): `g_faceMemory`: pointer (`scene::objects`, `race::stage_free`).
    pub face_memory: PlainDword,
    /// +0x660 (0x5C6FAC): `g_objectPointsRoom`: bytes left for the object points (`scene::objects`).
    pub object_points_room: PlainDword,
    /// +0x664 (0x5C6FB0): `g_faceMemoryRoom`: bytes left for the faces (`scene::objects`).
    pub face_memory_room: PlainDword,
    /// +0x668 (0x5C6FB4): `dword_5C6FB4`: set by the code at 0x41F7EF / 0x41F809 (unused by Rust).
    pub dword_6fb4_unused: PlainDword,
    /// +0x66c (0x5C6FB8): `g_pointRecords`: pointer to the point records (`scene::project_point`, `race::frame_draw`, `race::car_shadow`).
    pub point_records: PlainDword,
    /// +0x670 (0x5C6FBC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_670: [PlainDword; 2],
    /// +0x678 (0x5C6FC4): the original cleared it in `race::clear_point_records`; no code reads or writes it (the field's name nowhere in Rust), it only keeps the image's layout.
    pub unused_678: PlainDword,
    /// +0x67c (0x5C6FC8): `dword_5C6FC8`: the course memory's first part (`race::course_data`, `race::level_init`).
    pub first_part: PlainDword,
    /// +0x680 (0x5C6FCC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_680: PlainDword,
    /// +0x684 (0x5C6FD0): `flt_5C6FD0`: work of `scene::square_matrix`, row 0 (`scene::view_matrix`); now a local there; unused.
    pub squared_row_0: [PlainFloat; 4],
    /// +0x694 (0x5C6FE0): `flt_5C6FE0`: work of `scene::square_matrix`, row 1; now a local there; unused.
    pub squared_row_1: [PlainFloat; 4],
    /// +0x6a4 (0x5C6FF0): `flt_5C6FF0`: row 2 and the four dwords after, copied back (`scene::square_matrix`); now a local there; unused.
    pub squared_row_2_and_after: [PlainFloat; 8],
    /// +0x6c4 (0x5C7010): `flt_5C7010`: the turn matrix `physics::move_slot` and `physics::spin_matrix` make with `physics::axis_turn_matrix`.
    pub turn_matrix: [PlainFloat; 16],
    /// +0x704 (0x5C7050): `flt_5C7050`: the nine floats copied by `scene::invert_turn` (`scene::view_matrix`); no reader: unused.
    pub invert_copy: [PlainFloat; 9],
    /// +0x728 (0x5C7074): `flt_5C7074`: the nine floats made by `scene::invert_turn`; no reader: unused.
    pub invert_made: [PlainFloat; 9],
    /// +0x74c (0x5C7098): `flt_5C7098`: sine of the angle (`physics::axis_turn_matrix`). 0x5C7098 .. 0x5C70B8 are its work values.
    pub turn_sine: PlainFloat,
    /// +0x750 (0x5C709C): `flt_5C709C`: cosine (`physics::axis_turn_matrix`).
    pub turn_cosine: PlainFloat,
    /// +0x754 (0x5C70A0): `flt_5C70A0`: 1 - cosine.
    pub turn_one_less_cosine: PlainFloat,
    /// +0x758 (0x5C70A4): `flt_5C70A4`: (1 - cos) x z.
    pub turn_product_xz: PlainFloat,
    /// +0x75c (0x5C70A8): `flt_5C70A8`: (1 - cos) x y.
    pub turn_product_xy: PlainFloat,
    /// +0x760 (0x5C70AC): `flt_5C70AC`: (1 - cos) y z.
    pub turn_product_yz: PlainFloat,
    /// +0x764 (0x5C70B0): `flt_5C70B0`: x times sine.
    pub turn_sine_x: PlainFloat,
    /// +0x768 (0x5C70B4): `flt_5C70B4`: y times sine.
    pub turn_sine_y: PlainFloat,
    /// +0x76c (0x5C70B8): `flt_5C70B8`: z times sine.
    pub turn_sine_z: PlainFloat,
    /// +0x770 (0x5C70BC): `flt_5C70BC`: divisor in the unnormalised-vector function at 0x42115x (the vector's length).
    pub vector_length: PlainFloat,
    /// +0x774 (0x5C70C0): `flt_5C70C0`: the unit vector's x (the vector divided by `vector_length`), written by that function.
    pub vector_unit_x: PlainFloat,
    /// +0x778 (0x5C70C4): `flt_5C70C4`: the unit vector's y, as above.
    pub vector_unit_y: PlainFloat,
    /// +0x77c (0x5C70C8): `flt_5C70C8`: the unit vector's z, as above.
    pub vector_unit_z: PlainFloat,
    /// +0x780 (0x5C70CC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_780: PlainDword,
    /// +0x784 (0x5C70D0): `flt_5C70D0`: a scratch float stored at 0x4211A3 by the same function.
    pub vector_scratch: PlainFloat,
    /// +0x788 (0x5C70D4): `flt_5C70D4`: the car's pitch: minus the clamped asin of the turned forward vector's y (`physics::matrix_angles`, `physics::car_angles`).
    pub car_pitch: PlainFloat,
    /// +0x78c (0x5C70D8): `flt_5C70D8`: the car's heading: atan2 of the turned forward vector's x over z (`physics::matrix_angles`).
    pub car_heading: PlainFloat,
    /// +0x790 (0x5C70DC): `flt_5C70DC`: the car's roll: asin of the turned side vector's y over the pitch's cosine (`physics::matrix_angles`).
    pub car_roll: PlainFloat,
    /// +0x794 (0x5C70E0): `flt_5C70E0`: the turned unit vector's x (`physics::matrix_angles`).
    pub turned_x: PlainFloat,
    /// +0x798 (0x5C70E4): `flt_5C70E4`: its y.
    pub turned_y: PlainFloat,
    /// +0x79c (0x5C70E8): `flt_5C70E8`: its z.
    pub turned_z: PlainFloat,
    /// +0x7a0 (0x5C70EC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_7a0: [PlainDword; 3],
    /// +0x7ac (0x5C70F8): `g_rootTable`: square roots by mantissa bits (`math::normalise_vector`, `scene::fill_engine_tables`, `race::frame_draw`).
    pub root_table: [PlainFloat; 256],
    /// +0xbac (0x5C74F8): `g_exponentRootTable`: roots by exponent (same users).
    pub exponent_root_table: [PlainFloat; 256],
    /// +0xfac (0x5C78F8): `flt_5C78F8`: the angle of the turn (`physics::axis_turn_matrix`, `physics::spin_matrix`, `physics::move_slot`).
    pub spin_angle: PlainFloat,
    /// +0xfb0 (0x5C78FC): `flt_5C78FC` ..: the axis' x, y, z.
    pub spin_axis: [PlainFloat; 3],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
impl CourseLists {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x5C694C, the first byte of the piece.
pub const START: usize = 0x5C694C;
/// IDA 0x5C7908, the end of the piece.
pub const END: usize = 0x5C7908;

const _: () = assert!(END - START == core::mem::size_of::<CourseLists>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(crate::race::owner::RACE.course.lists.get()).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::race::owner::RACE.course.lists.get(),
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(CourseLists, input_keys_mask), 0x0);
        assert_eq!(offset_of!(CourseLists, list_6ed0), 0x584);
        assert_eq!(offset_of!(CourseLists, course_info), 0x654);
        assert_eq!(offset_of!(CourseLists, point_records), 0x66c);
        assert_eq!(offset_of!(CourseLists, turn_matrix), 0x6c4);
        assert_eq!(offset_of!(CourseLists, turn_product_xz), 0x758);
        assert_eq!(offset_of!(CourseLists, vector_length), 0x770);
        assert_eq!(offset_of!(CourseLists, car_heading), 0x78c);
        assert_eq!(offset_of!(CourseLists, exponent_root_table), 0xbac);
    }
}
