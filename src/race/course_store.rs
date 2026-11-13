//! The course's data and the scene's state (IDA 0x5CCBF0 .. 0x5CCE7C; phase 7, typed): the
//! course's records, faces and scene points (addresses and counts), the ground face being
//! probed, the faces' light, the camera's and start place's place and heading, a shape's
//! radius, the car model's part groups' masks, the shine. The fields are in the image's order. Plain cells: a drawing store (`session::DrawShared`), written on the game's thread
//! between frames and read by the drawing parts through their `workers::DrawReads`. Names are
//! readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CourseWork {
    /// +0x0 (0x5CCBF0): a course record's x and z (`race::frame_draw`, `race::car_shadow`); written only, by nothing now (a dead store); unused.
    pub record_place: [PlainDword; 2],
    /// +0x8 (0x5CCBF8): IDA writes it only in code outside any function (0x421C88 .. 0x421CDB, dead); the port never reaches it.
    pub unused_8: [PlainDword; 26],
    /// +0x70 (0x5CCC60): the ground face's corners (`physics::ground`).
    pub ground_corners: [PlainDword; 3],
    /// +0x7c (0x5CCC6C): (`physics::ground`, `physics::probe_ground`).
    pub ground_half: PlainDword,
    /// +0x80 (0x5CCC70): the ground face's number (`physics::ground`).
    pub ground_number: PlainDword,
    /// +0x84 (0x5CCC74): (`physics::camera_ground`, `physics::count_ground_probe`).
    pub ground_slot: PlainDword,
    /// +0x88 (0x5CCC78): the face found last time (`physics::camera_ground`).
    pub ground_face_last: PlainDword,
    /// +0x8c (0x5CCC7C): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_8c: [PlainDword; 5],
    /// +0xa0 (0x5CCC90): the face's four light bytes (`physics::car_lighting`).
    pub face_light_bytes: [PlainDword; 4],
    /// +0xb0 (0x5CCCA0): (`physics::car_lighting`).
    pub face_light_b: PlainDword,
    /// +0xb4 (0x5CCCA4): (`physics::car_lighting`).
    pub face_light_a: PlainDword,
    /// +0xb8 (0x5CCCA8): the car's light level kept (`physics::car_lighting`).
    pub light_kept: PlainDword,
    /// +0xbc (0x5CCCAC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_bc: [PlainDword; 3],
    /// +0xc8 (0x5CCCB8): `g_courseRecords`: the course records (an address).
    pub course_records: PlainDword,
    /// +0xcc (0x5CCCBC): `g_courseFaces`: the course faces (an address, 0x40 each).
    pub course_faces: PlainDword,
    /// +0xd0 (0x5CCCC0): `g_scenePoints`: the scene points (an address; x, y, z whole numbers, 12 bytes each).
    pub scene_points: PlainDword,
    /// +0xd4 (0x5CCCC4): the last two dwords of the course's layout (`race::norms`).
    pub course_tail: [PlainDword; 2],
    /// +0xdc (0x5CCCCC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_dc: PlainDword,
    /// +0xe0 (0x5CCCD0): `g_currentCourseFace` ("polynum"); now each view's
    /// (`race::face_work::FaceWork::course_face`), this one not used.
    pub current_course_face: PlainDword,
    /// +0xe4 (0x5CCCD4): `g_polygonFlags`: the list entry's flags word; nothing reads it to
    /// effect (`notes/bugs.md`), the port does not store it.
    pub polygon_flags: PlainDword,
    /// +0xe8 (0x5CCCD8): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_e8: [PlainDword; 15],
    /// +0x124 (0x5CCD14): `g_courseHeader` (`race::course_data`).
    pub course_header: PlainDword,
    /// +0x128 (0x5CCD18): `g_courseRecordCount`.
    pub course_record_count: PlainDword,
    /// +0x12c (0x5CCD1C): `g_courseFaceCount`.
    pub course_face_count: PlainDword,
    /// +0x130 (0x5CCD20): `g_scenePointCount`.
    pub scene_point_count: PlainDword,
    /// +0x134 (0x5CCD24): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_134: PlainDword,
    /// +0x138 (0x5CCD28): the start place's face (`race::start_place`).
    pub place_face: PlainDword,
    /// +0x13c (0x5CCD2C): IDA pushes it only in code outside any function (0x465ADF .. 0x465B03, dead); the port never reaches it.
    pub unused_13c: [PlainDword; 5],
    /// +0x150 (0x5CCD40): (`physics::update_camera`, `race::frame_draw`).
    pub camera_first: PlainDword,
    /// +0x154 (0x5CCD44): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_154: [PlainDword; 7],
    /// +0x170 (0x5CCD60): the camera's or start place's heading.
    pub place_heading: PlainDword,
    /// +0x174 (0x5CCD64): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_174: [PlainDword; 7],
    /// +0x190 (0x5CCD80): (`scene::project_point`).
    pub flat_depth: PlainDword,
    /// +0x194 (0x5CCD84): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_194: [PlainDword; 7],
    /// +0x1b0 (0x5CCDA0): the camera's or start place's x.
    pub place_x: PlainDword,
    /// +0x1b4 (0x5CCDA4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_1b4: [PlainDword; 7],
    /// +0x1d0 (0x5CCDC0): its y.
    pub place_y: PlainDword,
    /// +0x1d4 (0x5CCDC4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_1d4: [PlainDword; 7],
    /// +0x1f0 (0x5CCDE0): its z.
    pub place_z: PlainDword,
    /// +0x1f4 (0x5CCDE4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_1f4: [PlainDword; 12],
    /// +0x224 (0x5CCE14): (`scene::add_object_shape`).
    pub shape_radius: PlainDword,
    /// +0x228 (0x5CCE18): the moved point's x as the original stored it (`scene::place_object_points`, which stored y and z on over `group_masks[0]` and `[1]`); the drawing keeps the three as its own values: unused.
    pub point_moved: PlainDword,
    /// +0x22c (0x5CCE1C): the part groups' masks: here only `scene::load_object`'s (a load);
    /// the drawing's are the view's (`race::car_draw_state`).
    pub group_masks: [PlainDword; 18],
    /// +0x274 (0x5CCE64): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_274: [PlainDword; 4],
    /// +0x284 (0x5CCE74): `g_shine`; now each view's (`race::view_course::ViewCourse`); unused.
    pub shine: PlainDword,
    /// +0x288 (0x5CCE78): the shine as a whole number (`race::car_draw` stored it beside
    /// `shine`); read by nothing, not kept.
    pub shine_whole: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
impl CourseWork {
    pub const fn new() -> Self {
        Self {
            course_header: PlainDword::new(0x74756F73),
            camera_first: PlainDword::new(0x4000),
            shine: PlainDword::new(0x100),
            shine_whole: PlainDword::new(0x100),
            ..ZeroCells::ZERO
        }
    }
}
