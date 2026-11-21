//! The scene's matrices in the race block: the turned / lit matrices, the view matrices and the
//! object matrices. Most are 4x4 float matrices (64 bytes) the matrix functions take and return as
//! values (`scene::matrix::Matrix`); the callers store the results here. The identity and the
//! mirror work matrix, which nothing writes, are functions, and the turn and placed matrices,
//! whose unset floats never change, are the bases the turns start from
//! (`scene::matrix::TURN_X_BASE` ..). Plain cells: a drawing store
//! (`session::DrawShared`), written on the game's thread between frames and read by the drawing
//! parts through their `scene::workers::DrawReads`.
//!

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SceneMatrices {
    /// The model last set up for drawing (`scene::load_object`, `race::car_draw`).
    pub known_object: PlainDword,
    /// The car body's turn matrix copied from the car (`race::car_parts`, `race::car_draw`).
    pub body_turn_matrix: [PlainFloat; 16],
    /// `drive::set_listener` copies the made object matrix into it; `effects::draw_shadow` turns
    /// the shadow's corners by it.
    pub listener_matrix: [PlainFloat; 16],
    /// The turned matrix (`race::frame_draw`, `physics::move_slot`, `scene::matrix`).
    pub turned_matrix: [PlainFloat; 16],
    /// The lit matrix (`race::frame_draw`, `race::car_parts`).
    pub lit_matrix: [PlainFloat; 16],
    /// The view's place matrix: the copy of the placed matrix (the view's place negated) made in
    /// `race::frame_draw::view_place_tail`.
    pub view_place_matrix: [PlainFloat; 16],
    /// The view turned back (`scene::view_matrix`, `scene::reflect_vector`).
    pub view_back_matrix: [PlainFloat; 16],
    /// The view's turn (`drive::set_listener`, `physics::car_sounds`, `race::frame_draw`).
    pub view_turn_matrix: [PlainFloat; 16],
    /// The view matrix and its elements, used in place (`scene::project_point`,
    /// `effects::draw_particle`, `race::skidmarks`).
    pub view_matrix: [PlainFloat; 16],
    /// The matrix made for an object (`scene::matrix`, `race::frame_draw`).
    pub made_object_matrix: [PlainFloat; 16],
    /// The view's matrix, place (the last row of its 4 x 4 floats) (`effects::view_motion`, `race::frame_views`).
    pub view_place: [PlainFloat; 16],
    /// The object's turn matrix (`physics::object_turn_matrix`, `physics::object_hits_car`,
    /// `race::frame_draw`).
    pub object_turn_matrix: [PlainFloat; 16],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The 4x4 identity.
pub fn identity_matrix() -> [PlainFloat; 16] {
    let mut a: [PlainFloat; 16] = ZeroCells::ZERO;
    a[0] = PlainFloat::new(1.0);
    a[5] = PlainFloat::new(1.0);
    a[10] = PlainFloat::new(1.0);
    a[15] = PlainFloat::new(1.0);
    a
}

/// The constant matrix the frame draw turns a mirrored view's car by: the diagonal (-1, 1, -1, 1).
pub fn mirror_work_matrix() -> [PlainFloat; 16] {
    let mut a: [PlainFloat; 16] = ZeroCells::ZERO;
    a[0] = PlainFloat::new(-1.0);
    a[5] = PlainFloat::new(1.0);
    a[10] = PlainFloat::new(-1.0);
    a[15] = PlainFloat::new(1.0);
    a
}

/// The piece, with its first values.
impl SceneMatrices {
    pub const fn new() -> Self {
        Self::ZERO
    }
}
