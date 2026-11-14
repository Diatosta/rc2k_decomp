//! The course's data and the scene's state: the course's records, faces and scene points (counts
//! and places), the ground face being probed, the faces' light, the camera's and start place's
//! place and heading, a shape's radius, the car model's part groups' masks, the shine. The fields
//! are in the image's order. Plain cells: a drawing store (`session::DrawShared`), written on the
//! game's thread between frames and read by the drawing parts through their `workers::DrawReads`.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CourseWork {
    /// +0x0: a course record's x and z (`race::frame_draw`, `race::car_shadow`); written by
    /// nothing; unused.
    pub record_place: [PlainDword; 2],
    /// +0x8: unused.
    pub unused_8: [PlainDword; 26],
    /// +0x70: the ground face's corners (`physics::ground`).
    pub ground_corners: [PlainDword; 3],
    /// +0x7c: (`physics::ground`, `physics::probe_ground`).
    pub ground_half: PlainDword,
    /// +0x80: the ground face's number (`physics::ground`).
    pub ground_number: PlainDword,
    /// +0x84: (`physics::camera_ground`, `physics::count_ground_probe`).
    pub ground_slot: PlainDword,
    /// +0x88: the face found last time (`physics::camera_ground`).
    pub ground_face_last: PlainDword,
    /// +0x8c: unused; it only keeps the image's layout.
    pub unused_8c: [PlainDword; 5],
    /// +0xa0: the face's four light bytes (`physics::car_lighting`).
    pub face_light_bytes: [PlainDword; 4],
    /// +0xb0: (`physics::car_lighting`).
    pub face_light_b: PlainDword,
    /// +0xb4: (`physics::car_lighting`).
    pub face_light_a: PlainDword,
    /// +0xb8: the car's light level kept (`physics::car_lighting`).
    pub light_kept: PlainDword,
    /// +0xbc: unused; it only keeps the image's layout.
    pub unused_bc: [PlainDword; 3],
    /// +0xc8: the course records.
    pub course_records: PlainDword,
    /// +0xcc: the course faces (0x40 bytes each).
    pub course_faces: PlainDword,
    /// +0xd0: the scene points (x, y, z whole numbers, 12 bytes each).
    pub scene_points: PlainDword,
    /// +0xd4: unused; it only keeps the image's layout.
    pub unused_d4: [PlainDword; 2],
    /// +0xdc: unused; it only keeps the image's layout.
    pub unused_dc: PlainDword,
    /// +0xe0: the face number ("polynum"); now each view's
    /// (`race::face_work::FaceWork::course_face`), this one not used.
    pub current_course_face: PlainDword,
    /// +0xe4: the list entry's flags word; nothing reads it to effect, so it is not stored.
    pub polygon_flags: PlainDword,
    /// +0xe8: unused; it only keeps the image's layout.
    pub unused_e8: [PlainDword; 15],
    /// +0x124: (`race::course_data`).
    pub course_header: PlainDword,
    /// +0x128: `course_record_count`.
    pub course_record_count: PlainDword,
    /// +0x12c: `course_face_count`.
    pub course_face_count: PlainDword,
    /// +0x130: `scene_point_count`.
    pub scene_point_count: PlainDword,
    /// +0x134: unused; it only keeps the image's layout.
    pub unused_134: PlainDword,
    /// +0x138: the start place's face (`race::start_place`).
    pub place_face: PlainDword,
    /// +0x13c: unused.
    pub unused_13c: [PlainDword; 5],
    /// +0x150: (`physics::update_camera`, `race::frame_draw`).
    pub camera_first: PlainDword,
    /// +0x154: unused; it only keeps the image's layout.
    pub unused_154: [PlainDword; 7],
    /// +0x170: the camera's or start place's heading.
    pub place_heading: PlainDword,
    /// +0x174: unused; it only keeps the image's layout.
    pub unused_174: [PlainDword; 7],
    /// +0x190: (`scene::project_point`).
    pub flat_depth: PlainDword,
    /// +0x194: unused; it only keeps the image's layout.
    pub unused_194: [PlainDword; 7],
    /// +0x1b0: the camera's or start place's x.
    pub place_x: PlainDword,
    /// +0x1b4: unused; it only keeps the image's layout.
    pub unused_1b4: [PlainDword; 7],
    /// +0x1d0: its y.
    pub place_y: PlainDword,
    /// +0x1d4: unused; it only keeps the image's layout.
    pub unused_1d4: [PlainDword; 7],
    /// +0x1f0: its z.
    pub place_z: PlainDword,
    /// +0x1f4: unused; it only keeps the image's layout.
    pub unused_1f4: [PlainDword; 12],
    /// +0x224: (`scene::add_object_shape`).
    pub shape_radius: PlainDword,
    /// +0x228: the moved point's x as first stored (`scene::place_object_points`, which stored y
    /// and z on over `group_masks[0]` and `[1]`); the drawing keeps the three as its own values:
    /// unused.
    pub point_moved: PlainDword,
    /// +0x22c: the part groups' masks: here only `scene::load_object`'s (a load); the drawing's are
    /// the view's (`race::car_draw_state`).
    pub group_masks: [PlainDword; 18],
    /// +0x274: unused; it only keeps the image's layout.
    pub unused_274: [PlainDword; 4],
    /// +0x284: `shine`; now each view's (`race::view_course::ViewCourse`); unused.
    pub shine: PlainDword,
    /// +0x288: the shine as a whole number (`race::car_draw` stored it beside `shine`); read by
    /// nothing, not kept.
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
