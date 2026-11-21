//! The course's counts and the camera's or start place's place: how many records, faces and
//! scene points the course holds, the place and heading the views start from, and the flat
//! stages' depth. A plain record (`session::DrawPlain`): written on the game's thread between
//! frames and read there and, as a copy made when a drawing scope opens, by the drawing parts.

use crate::memory::session::{DrawPlain, PlainRead};

/// The course's counts and places.
#[derive(Clone, Copy)]
pub struct CourseWork {
    /// The course's records, from the course file's header.
    pub course_record_count: u32,
    /// The course's faces, from the course file's header.
    pub course_face_count: u32,
    /// The course's scene points, from the course file's header.
    pub scene_point_count: u32,
    /// The camera's first figure (`physics::update_camera`; a view's turn).
    pub camera_first: u32,
    /// The camera's or start place's heading.
    pub place_heading: u32,
    /// The camera's distance to its target, for a flat stage (`physics::update_camera`).
    pub flat_depth: u32,
    /// The camera's or start place's x.
    pub place_x: u32,
    /// Its y.
    pub place_y: u32,
    /// Its z.
    pub place_z: u32,
}

impl CourseWork {
    pub const fn new() -> Self {
        Self {
            course_record_count: 0,
            course_face_count: 0,
            scene_point_count: 0,
            camera_first: 0x4000,
            place_heading: 0,
            flat_depth: 0,
            place_x: 0,
            place_y: 0,
            place_z: 0,
        }
    }
}

/// The counts and places as one reader finds them: the game's thread's own (read through the
/// store when used), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum CourseWorkRef<'a> {
    Store(&'a DrawPlain<CourseWork>),
    Copy(&'a CourseWork),
}

impl CourseWorkRef<'_> {
    /// The record as it stands.
    pub fn get(self) -> CourseWork {
        match self {
            Self::Store(store) => *store.borrow(),
            Self::Copy(record) => *record,
        }
    }
}

impl<'a> From<&'a PlainRead<CourseWork>> for CourseWorkRef<'a> {
    fn from(copy: &'a PlainRead<CourseWork>) -> Self {
        Self::Copy(copy)
    }
}
