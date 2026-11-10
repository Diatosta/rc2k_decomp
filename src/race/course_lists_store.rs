//! The piece from the input keys word 0x5C694C to the spin axis at 0x5C7904: the face-key factors, the course memory's lists and pointers, the object and point-record pointers, the matrix work areas (squaring, turn, inversion, angles), the two root tables and the spin variables.
//! It stops before the car records at 0x5C790C (`race::car_specs_store`).
//! Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CourseLists {
    /// +0x0 (0x5C694C): the keys read for the car's controls (`drive::read_car_controls`, `dword_5C694C`; a reading).
    pub input_keys_mask: Dword,
    /// +0x4 (0x5C6950): `g_faceKeyFactors`: floats for the face sort key (`scene::face_sort_key`, `scene::fill_engine_tables`).
    pub face_key_factors: [Float; 256],
    /// +0x404 (0x5C6D50): `g_courseMemoryList`, a MemoryList of 16 bytes (`race::allocate_course_memory`, `race::stage_free`).
    pub course_memory_list: [Dword; 4],
    /// +0x414 (0x5C6D60): not named.
    pub unknown_414: [Dword; 12],
    /// +0x444 (0x5C6D90): `stru_5C6D90`, a MemoryList (`race::free_course_list_2`, `race::stage_free`).
    pub course_list_2: [Dword; 4],
    /// +0x454 (0x5C6DA0): not named.
    pub unknown_454: [Dword; 12],
    /// +0x484 (0x5C6DD0): `stru_5C6DD0`, a MemoryList (`scene::allocate_object_memory`).
    pub object_points_list: [Dword; 4],
    /// +0x494 (0x5C6DE0): not named.
    pub unknown_494: [Dword; 28],
    /// +0x504 (0x5C6E50): `stru_5C6E50`, a MemoryList (`scene::allocate_point_records`).
    pub point_records_list: [Dword; 4],
    /// +0x514 (0x5C6E60): not named.
    pub unknown_514: [Dword; 28],
    /// +0x584 (0x5C6ED0): `stru_5C6ED0`, a MemoryList used by 0x459764, 0x4599D4 (not used by Rust;
    /// a reading).
    pub list_6ed0: [Dword; 4],
    /// +0x594 (0x5C6EE0): not named.
    pub unknown_594: [Dword; 28],
    /// +0x604 (0x5C6F50): `stru_5C6F50`, the mud's MemoryList (`race::mud`).
    pub mud_list: [Dword; 4],
    /// +0x614 (0x5C6F60): not named.
    pub unknown_614: [Dword; 12],
    /// +0x644 (0x5C6F90): `dword_5C6F90`: bytes taken from the course lists, added and subtracted by the allocators (`race::stage_free`, `race::mud`, `scene::allocate_object_memory`).
    pub course_bytes_total: Dword,
    /// +0x648 (0x5C6F94): `g_courseMemory`: handle of the course's memory, `level_init::COURSE_TAG` while taken (`race::course_data`, `race::norms`, `race::pace_notes`).
    pub course_memory: Dword,
    /// +0x64c (0x5C6F98): `dword_5C6F98`, set by `race::free_course_list_2` (cleared by `race::stage_free`).
    pub course_list_2_a: Dword,
    /// +0x650 (0x5C6F9C): `dword_5C6F9C`, as its neighbour (cleared by `race::stage_free`).
    pub course_list_2_b: Dword,
    /// +0x654 (0x5C6FA0): `g_courseInfo`: pointer to the last 0x400000 bytes of the course memory (`race::level_init`, `effects::profiles`, `effects::draw_frame`).
    pub course_info: Dword,
    /// +0x658 (0x5C6FA4): `g_objectPoints`: pointer to the object points (`scene::objects`, `scene::place_object_points`, `physics::object_hits_car`).
    pub object_points: Dword,
    /// +0x65c (0x5C6FA8): `g_faceMemory`: pointer (`scene::objects`, `race::stage_free`).
    pub face_memory: Dword,
    /// +0x660 (0x5C6FAC): `g_objectPointsRoom`: bytes left for the object points (`scene::objects`).
    pub object_points_room: Dword,
    /// +0x664 (0x5C6FB0): `g_faceMemoryRoom`: bytes left for the faces (`scene::objects`).
    pub face_memory_room: Dword,
    /// +0x668 (0x5C6FB4): `dword_5C6FB4`: set by the code at 0x41F7EF / 0x41F809 (a reading, unused by Rust).
    pub dword_6fb4_unused: Dword,
    /// +0x66c (0x5C6FB8): `g_pointRecords`: pointer to the point records (`scene::project_point`, `race::frame_draw`, `race::car_shadow`).
    pub point_records: Dword,
    /// +0x670 (0x5C6FBC): not named.
    pub unknown_670: [Dword; 2],
    /// +0x678 (0x5C6FC4): `dword_5C6FC4`: cleared by `race::clear_point_records` (`race::frame_draw`).
    pub clear_count: Dword,
    /// +0x67c (0x5C6FC8): `dword_5C6FC8`: the course memory's first part (`race::course_data`, `race::level_init`).
    pub first_part: Dword,
    /// +0x680 (0x5C6FCC): not named.
    pub unknown_680: Dword,
    /// +0x684 (0x5C6FD0): `flt_5C6FD0`: work of `scene::square_matrix`, row 0 (`scene::view_matrix`); now a local there; unused.
    pub squared_row_0: [Float; 4],
    /// +0x694 (0x5C6FE0): `flt_5C6FE0`: work of `scene::square_matrix`, row 1; now a local there; unused.
    pub squared_row_1: [Float; 4],
    /// +0x6a4 (0x5C6FF0): `flt_5C6FF0`: row 2 and the four dwords after, copied back (`scene::square_matrix`); now a local there; unused.
    pub squared_row_2_and_after: [Float; 8],
    /// +0x6c4 (0x5C7010): `flt_5C7010`: the turn matrix `physics::move_slot` and `physics::spin_matrix` make with `physics::axis_turn_matrix`.
    pub turn_matrix: [Float; 16],
    /// +0x704 (0x5C7050): `flt_5C7050`: the nine floats copied by `scene::invert_turn` (`scene::view_matrix`); no reader: unused.
    pub invert_copy: [Float; 9],
    /// +0x728 (0x5C7074): `flt_5C7074`: the nine floats made by `scene::invert_turn`; no reader: unused.
    pub invert_made: [Float; 9],
    /// +0x74c (0x5C7098): `flt_5C7098`: sine of the angle (`physics::axis_turn_matrix`). 0x5C7098 .. 0x5C70B8 are its work values.
    pub turn_sine: Float,
    /// +0x750 (0x5C709C): `flt_5C709C`: cosine (`physics::axis_turn_matrix`).
    pub turn_cosine: Float,
    /// +0x754 (0x5C70A0): `flt_5C70A0`: 1 - cosine.
    pub turn_one_less_cosine: Float,
    /// +0x758 (0x5C70A4): `flt_5C70A4`: (1 - cos) x z.
    pub turn_product_xz: Float,
    /// +0x75c (0x5C70A8): `flt_5C70A8`: (1 - cos) x y.
    pub turn_product_xy: Float,
    /// +0x760 (0x5C70AC): `flt_5C70AC`: (1 - cos) y z.
    pub turn_product_yz: Float,
    /// +0x764 (0x5C70B0): `flt_5C70B0`: x times sine.
    pub turn_sine_x: Float,
    /// +0x768 (0x5C70B4): `flt_5C70B4`: y times sine.
    pub turn_sine_y: Float,
    /// +0x76c (0x5C70B8): `flt_5C70B8`: z times sine.
    pub turn_sine_z: Float,
    /// +0x770 (0x5C70BC): `flt_5C70BC`: divisor in the unnormalised-vector function at 0x42115x (a reading: a length).
    pub vector_length: Float,
    /// +0x774 (0x5C70C0): `flt_5C70C0`: written by that function (a reading).
    pub vector_unit_x: Float,
    /// +0x778 (0x5C70C4): `flt_5C70C4`: as above (a reading).
    pub vector_unit_y: Float,
    /// +0x77c (0x5C70C8): `flt_5C70C8`: as above (a reading).
    pub vector_unit_z: Float,
    /// +0x780 (0x5C70CC): not named.
    pub unknown_780: Dword,
    /// +0x784 (0x5C70D0): `flt_5C70D0`: stored at 0x4211A3 (a reading).
    pub vector_scratch: Float,
    /// +0x788 (0x5C70D4): `flt_5C70D4`: minus asin (`physics::matrix_angles`, `physics::car_angles`; a reading).
    pub car_pitch: Float,
    /// +0x78c (0x5C70D8): `flt_5C70D8`: fpatan result (`physics::matrix_angles`; a reading).
    pub car_heading: Float,
    /// +0x790 (0x5C70DC): `flt_5C70DC`: (`physics::matrix_angles`; a reading).
    pub car_roll: Float,
    /// +0x794 (0x5C70E0): `flt_5C70E0`: the turned unit vector's x (`physics::matrix_angles`).
    pub turned_x: Float,
    /// +0x798 (0x5C70E4): `flt_5C70E4`: its y.
    pub turned_y: Float,
    /// +0x79c (0x5C70E8): `flt_5C70E8`: its z.
    pub turned_z: Float,
    /// +0x7a0 (0x5C70EC): not named.
    pub unknown_7a0: [Dword; 3],
    /// +0x7ac (0x5C70F8): `g_rootTable`: square roots by mantissa bits (`math::normalise_vector`, `scene::fill_engine_tables`, `race::frame_draw`).
    pub root_table: [Float; 256],
    /// +0xbac (0x5C74F8): `g_exponentRootTable`: roots by exponent (same users).
    pub exponent_root_table: [Float; 256],
    /// +0xfac (0x5C78F8): `flt_5C78F8`: the angle of the turn (`physics::axis_turn_matrix`, `physics::spin_matrix`, `physics::move_slot`).
    pub spin_angle: Float,
    /// +0xfb0 (0x5C78FC): `flt_5C78FC` ..: the axis' x, y, z.
    pub spin_axis: [Float; 3],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a field of `race::owner::Race::course`, formerly a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl CourseLists {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x5C694C, the first byte of the piece.
pub const START: usize = 0x5C694C;
/// IDA 0x5C7908, the end of the piece.
pub const END: usize = 0x5C7908;

const _: () = assert!(END - START == core::mem::size_of::<CourseLists>());

/// Where IDA 0x5C694C is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&crate::race::owner::RACE.course.lists))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&crate::race::owner::RACE.course.lists, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(CourseLists, input_keys_mask), 0x5C694C - START);
        assert_eq!(offset_of!(CourseLists, list_6ed0), 0x5C6ED0 - START);
        assert_eq!(offset_of!(CourseLists, course_info), 0x5C6FA0 - START);
        assert_eq!(offset_of!(CourseLists, point_records), 0x5C6FB8 - START);
        assert_eq!(offset_of!(CourseLists, turn_matrix), 0x5C7010 - START);
        assert_eq!(offset_of!(CourseLists, turn_product_xz), 0x5C70A4 - START);
        assert_eq!(offset_of!(CourseLists, vector_length), 0x5C70BC - START);
        assert_eq!(offset_of!(CourseLists, car_heading), 0x5C70D8 - START);
        assert_eq!(
            offset_of!(CourseLists, exponent_root_table),
            0x5C74F8 - START
        );
    }
}
