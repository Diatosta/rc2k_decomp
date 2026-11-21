//! The frame's work vector: three floats the contact steps and the faces' normals work in place
//! (`race::normalise_work_vector`), kept between them as bits.

/// The work vector.
pub struct FrameScratch {
    /// Three floats' bits (`physics::face_contact`, `physics::wall_push_back`, `race::norms`,
    /// `drive::drive_computer_car`); taken in place.
    pub work_vector: [u32; 3],
}

impl FrameScratch {
    pub const fn new() -> Self {
        Self {
            work_vector: [0; 3],
        }
    }
}
