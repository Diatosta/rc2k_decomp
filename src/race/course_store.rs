//! The course's data and the scene's state (IDA 0x5CCBF0 .. 0x5CCE7C; phase 7, typed): the
//! course's records, faces and scene points (addresses and counts), the ground face being
//! probed, the faces' light, the camera's and start place's place and heading, a shape's
//! radius, the car model's part groups' masks, the shine. `#[repr(C)]` with the image's
//! layout, over the race block's bytes (`COURSE`), as `CAR`. Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CourseWork {
    /// +0x0 (0x5CCBF0): a course record's x and z (`race::frame_draw`, `race::car_shadow`); written only, by nothing now (a dead store); unused.
    pub record_place: [Dword; 2],
    /// +0x8 (0x5CCBF8): not named.
    pub unknown_8: [Dword; 26],
    /// +0x70 (0x5CCC60): the ground face's corners (`physics::ground`).
    pub ground_corners: [Dword; 3],
    /// +0x7c (0x5CCC6C): (`physics::ground`, `physics::probe_ground`).
    pub ground_half: Dword,
    /// +0x80 (0x5CCC70): the ground face's number (`physics::ground`).
    pub ground_number: Dword,
    /// +0x84 (0x5CCC74): (`physics::camera_ground`, `physics::count_ground_probe`).
    pub ground_slot: Dword,
    /// +0x88 (0x5CCC78): the face found last time (`physics::camera_ground`).
    pub ground_face_last: Dword,
    /// +0x8c (0x5CCC7C): not named.
    pub unknown_8c: [Dword; 5],
    /// +0xa0 (0x5CCC90): the face's four light bytes (`physics::car_lighting`).
    pub face_light_bytes: [Dword; 4],
    /// +0xb0 (0x5CCCA0): (`physics::car_lighting`).
    pub face_light_b: Dword,
    /// +0xb4 (0x5CCCA4): (`physics::car_lighting`).
    pub face_light_a: Dword,
    /// +0xb8 (0x5CCCA8): the car's light level kept (`physics::car_lighting`).
    pub light_kept: Dword,
    /// +0xbc (0x5CCCAC): not named.
    pub unknown_bc: [Dword; 3],
    /// +0xc8 (0x5CCCB8): `g_courseRecords`: the course records (an address).
    pub course_records: Dword,
    /// +0xcc (0x5CCCBC): `g_courseFaces`: the course faces (an address, 0x40 each).
    pub course_faces: Dword,
    /// +0xd0 (0x5CCCC0): `g_scenePoints`: the scene points (an address; x, y, z whole numbers, 12 bytes each).
    pub scene_points: Dword,
    /// +0xd4 (0x5CCCC4): the last two dwords of the course's layout (`race::norms`).
    pub course_tail: [Dword; 2],
    /// +0xdc (0x5CCCCC): not named.
    pub unknown_dc: Dword,
    /// +0xe0 (0x5CCCD0): `g_currentCourseFace` ("polynum"); now each view's
    /// (`race::face_work::FaceWork::course_face`), this one not used.
    pub current_course_face: Dword,
    /// +0xe4 (0x5CCCD4): `g_polygonFlags`: the list entry's flags word; nothing reads it to
    /// effect (`notes/bugs.md`), the port does not store it.
    pub polygon_flags: Dword,
    /// +0xe8 (0x5CCCD8): not named.
    pub unknown_e8: [Dword; 15],
    /// +0x124 (0x5CCD14): `g_courseHeader` (`race::course_data`).
    pub course_header: Dword,
    /// +0x128 (0x5CCD18): `g_courseRecordCount`.
    pub course_record_count: Dword,
    /// +0x12c (0x5CCD1C): `g_courseFaceCount`.
    pub course_face_count: Dword,
    /// +0x130 (0x5CCD20): `g_scenePointCount`.
    pub scene_point_count: Dword,
    /// +0x134 (0x5CCD24): not named.
    pub unknown_134: Dword,
    /// +0x138 (0x5CCD28): the start place's face (`race::start_place`).
    pub place_face: Dword,
    /// +0x13c (0x5CCD2C): not named.
    pub unknown_13c: [Dword; 5],
    /// +0x150 (0x5CCD40): (`physics::update_camera`, `race::frame_draw`).
    pub camera_first: Dword,
    /// +0x154 (0x5CCD44): not named.
    pub unknown_154: [Dword; 7],
    /// +0x170 (0x5CCD60): the camera's or start place's heading.
    pub place_heading: Dword,
    /// +0x174 (0x5CCD64): not named.
    pub unknown_174: [Dword; 7],
    /// +0x190 (0x5CCD80): (`scene::project_point`).
    pub flat_depth: Dword,
    /// +0x194 (0x5CCD84): not named.
    pub unknown_194: [Dword; 7],
    /// +0x1b0 (0x5CCDA0): the camera's or start place's x.
    pub place_x: Dword,
    /// +0x1b4 (0x5CCDA4): not named.
    pub unknown_1b4: [Dword; 7],
    /// +0x1d0 (0x5CCDC0): its y.
    pub place_y: Dword,
    /// +0x1d4 (0x5CCDC4): not named.
    pub unknown_1d4: [Dword; 7],
    /// +0x1f0 (0x5CCDE0): its z.
    pub place_z: Dword,
    /// +0x1f4 (0x5CCDE4): not named.
    pub unknown_1f4: [Dword; 12],
    /// +0x224 (0x5CCE14): (`scene::add_object_shape`).
    pub shape_radius: Dword,
    /// +0x228 (0x5CCE18): the moved point's x (`scene::move_point_between`, which stores y and z on over `group_masks[0]` and `[1]`).
    pub point_moved: Dword,
    /// +0x22c (0x5CCE1C): the part groups' masks: here only `scene::load_object`'s (a load) and
    /// the moved point's y and z over [0], [1]; the drawing's are the view's
    /// (`race::car_draw_state`).
    pub group_masks: [Dword; 18],
    /// +0x274 (0x5CCE64): not named.
    pub unknown_274: [Dword; 4],
    /// +0x284 (0x5CCE74): `g_shine`; now each view's (`race::view_course::ViewCourse`); unused.
    pub shine: Dword,
    /// +0x288 (0x5CCE78): the shine as a whole number (`race::car_draw` stored it beside
    /// `shine`); read by nothing, not kept.
    pub shine_whole: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a field of `race::owner::Race::course`, formerly a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl CourseWork {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x5CCBF0, the first byte of the piece.
pub const START: usize = 0x5CCBF0;
/// IDA 0x5CCE7C, the end of the piece.
pub const END: usize = 0x5CCE7C;

const _: () = assert!(END - START == core::mem::size_of::<CourseWork>());

/// Where IDA 0x5CCBF0 is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&crate::race::owner::RACE.course.course))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&crate::race::owner::RACE.course.course, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(CourseWork, record_place), 0x5CCBF0 - START);
        assert_eq!(offset_of!(CourseWork, face_light_bytes), 0x5CCC90 - START);
        assert_eq!(offset_of!(CourseWork, scene_points), 0x5CCCC0 - START);
        assert_eq!(offset_of!(CourseWork, course_face_count), 0x5CCD1C - START);
        assert_eq!(offset_of!(CourseWork, place_x), 0x5CCDA0 - START);
        assert_eq!(offset_of!(CourseWork, shine), 0x5CCE74 - START);
    }
}
