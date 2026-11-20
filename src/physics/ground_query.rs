//! What a ground search (`physics::ground`) reads and writes: one plain record of values
//! (`GroundSearch`) that the physics keeps for the game's thread (`race::owner_physics::Physics`)
//! and each drawing part keeps as a local of its own (the shadow's and the course view's searches
//! do not touch the physics' record, since nothing in the physics reads what they leave there),
//! and the inputs the search only reads (`GroundEnv`).

use crate::race::owner_course::CourseReads;
use crate::race::scene_points::OwnPoints;

/// The search's fields (the physics' names in brackets), all 0 to begin with.
#[derive(Clone, Debug, Default)]
pub struct GroundSearch {
    /// The place asked for (`ground_x`, `ground_z`).
    pub x: u32,
    pub z: u32,
    /// The ground's height under it (`ground_height`), 1 when a face was found
    /// (`ground_found`).
    pub height: u32,
    pub found: u32,
    /// The face's three corner point numbers (`face_corner_numbers`), their places
    /// (`face_corners`).
    pub corner_numbers: [u32; 3],
    pub corners: [u32; 9],
    /// `physics::face_shares`' outputs a later step reads: the two shares as floats (`plane_f`,
    /// `plane_g`) and as whole numbers (`share_s`, `share_t`).
    pub plane_f: f32,
    pub plane_g: f32,
    pub share_s: u32,
    pub share_t: u32,
    /// The face found (`ground_slot`), its half (`ground_half`) and the face tried first
    /// (`ground_face_last`).
    pub slot: u32,
    pub half: u32,
    pub last: u32,
}

impl GroundSearch {
    /// All 0, as a race begins.
    pub const fn new() -> Self {
        GroundSearch {
            x: 0,
            z: 0,
            height: 0,
            found: 0,
            corner_numbers: [0; 3],
            corners: [0; 9],
            plane_f: 0.0,
            plane_g: 0.0,
            share_s: 0,
            share_t: 0,
            slot: 0,
            half: 0,
            last: 0,
        }
    }
}

/// What the search only reads: the course, the scene points the faces' corners are read through
/// (a drawing part's own past the count in the drawing, `race::scene_points::OwnPoints`), and
/// the caller's mark (`settle_mark`, 0x40 or 0x80) that picks which faces count.
#[derive(Clone, Copy)]
pub struct GroundEnv<'a> {
    pub course: CourseReads<'a>,
    pub points: &'a OwnPoints,
    pub mark: u32,
}
