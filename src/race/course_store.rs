//! The course's data and the scene's state: the course's records, faces and scene points (counts
//! and places), the ground face being probed, the faces' light, the camera's and start place's
//! place and heading, a shape's radius, the car model's part groups' masks, the shine. The fields
//! are in the image's order. Plain cells: a drawing store (`session::DrawShared`), written on the
//! game's thread between frames and read by the drawing parts through their `scene::workers::DrawReads`.

use crate::memory::cells::ZeroCells;
use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CourseWork {
    /// A course record's x and z (`race::frame_draw`, `race::car_shadow`); written by
    /// nothing; unused.
    pub record_place: [PlainDword; 2],
    /// Unused.
    pub unused_8: [PlainDword; 26],
    /// The ground face's corners (`physics::ground`).
    pub ground_corners: [PlainDword; 3],
    /// (`physics::ground`, `physics::probe_ground`).
    pub ground_half: PlainDword,
    /// The ground face's number (`physics::ground`).
    pub ground_number: PlainDword,
    /// (`physics::camera_ground`, `physics::count_ground_probe`).
    pub ground_slot: PlainDword,
    /// The face found last time (`physics::camera_ground`).
    pub ground_face_last: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_8c: [PlainDword; 5],
    /// The face's four light bytes (`physics::car_lighting`).
    pub face_light_bytes: [PlainDword; 4],
    /// (`physics::car_lighting`).
    pub face_light_b: PlainDword,
    /// (`physics::car_lighting`).
    pub face_light_a: PlainDword,
    /// The car's light level kept (`physics::car_lighting`).
    pub light_kept: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_bc: [PlainDword; 3],
    /// Unused; it only keeps the image's layout.
    pub unused_dc: PlainDword,
    /// The face number ("polynum"); now each view's
    /// (`race::face_work::FaceWork::course_face`), this one not used.
    pub current_course_face: PlainDword,
    /// The list entry's flags word; nothing reads it to effect, so it is not stored.
    pub polygon_flags: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_e8: [PlainDword; 15],
    /// (`race::course_data`).
    pub course_header: PlainDword,
    /// `course_record_count`.
    pub course_record_count: PlainDword,
    /// `course_face_count`.
    pub course_face_count: PlainDword,
    /// `scene_point_count`.
    pub scene_point_count: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_134: PlainDword,
    /// The start place's face (`race::start_place`).
    pub place_face: PlainDword,
    /// Unused.
    pub unused_13c: [PlainDword; 5],
    /// (`physics::update_camera`, `race::frame_draw`).
    pub camera_first: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_154: [PlainDword; 7],
    /// The camera's or start place's heading.
    pub place_heading: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_174: [PlainDword; 7],
    /// (`scene::project_point`).
    pub flat_depth: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_194: [PlainDword; 7],
    /// The camera's or start place's x.
    pub place_x: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_1b4: [PlainDword; 7],
    /// Its y.
    pub place_y: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_1d4: [PlainDword; 7],
    /// Its z.
    pub place_z: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub unused_1f4: [PlainDword; 12],
    /// (`scene::add_object_shape`).
    pub shape_radius: PlainDword,
    /// The moved point's x as first stored (`scene::place_object_points`, which stored y
    /// and z on over `group_masks[0]` and `[1]`); the drawing keeps the three as its own values:
    /// unused.
    pub point_moved: PlainDword,
    /// The part groups' masks: here only `scene::load_object`'s (a load); the drawing's are
    /// the view's (`race::car_draw_state`).
    pub group_masks: [PlainDword; 18],
    /// Unused; it only keeps the image's layout.
    pub unused_274: [PlainDword; 4],
    /// `shine`; now each view's (`race::view_course::ViewCourse`); unused.
    pub shine: PlainDword,
    /// The shine as a whole number (`race::car_draw` stored it beside `shine`); read by
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
