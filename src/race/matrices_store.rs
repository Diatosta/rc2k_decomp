//! The scene's matrices in the race block: the identity, the three turn matrices, the placed /
//! turned / lit matrices, the view matrices, the object matrices and a scratch area. Most are 4x4
//! float matrices (64 bytes) passed to the matrix functions in place. One item is 1092 bytes,
//! taken whole; only its first 64 bytes are used. Plain cells: a drawing store
//! (`session::DrawShared`), written on the game's thread between frames and read by the drawing
//! parts through their `scene::workers::DrawReads`.
//!

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SceneMatrices {
    /// The model last set up for drawing (`scene::load_object`, `race::car_draw`).
    pub known_object: PlainDword,
    /// Two matrices of the frame draw, used in place as scratch (`race::frame_draw`).
    pub view_work: [PlainFloat; 32],
    /// The 4x4 identity (`scene::view_matrix`, `race::frame_draw`).
    pub identity_matrix: [PlainFloat; 16],
    /// Rotation about x (`scene::matrix`, `physics::object_turn_matrix`).
    pub turn_x_matrix: [PlainFloat; 16],
    /// Rotation about y (`scene::matrix`, `race::car_parts`).
    pub turn_y_matrix: [PlainFloat; 16],
    /// Rotation about z (`scene::view_matrix`, `race::car_draw`).
    pub turn_z_matrix: [PlainFloat; 16],
    /// A matrix with its place (`scene::matrix`, `physics::car_collisions`).
    pub placed_matrix: [PlainFloat; 16],
    /// The car body's turn matrix copied from the car (`race::car_parts`, `race::car_draw`).
    pub body_turn_matrix: [PlainFloat; 16],
    /// `drive::set_listener` copies the made object matrix into it; `effects::draw_shadow` turns
    /// the shadow's corners by it.
    pub listener_matrix: [PlainFloat; 16],
    /// The turned matrix (`race::frame_draw`, `physics::move_slot`, `scene::matrix`).
    pub turned_matrix: [PlainFloat; 16],
    /// The lit matrix (`race::frame_draw`, `race::car_parts`).
    pub lit_matrix: [PlainFloat; 16],
    /// The inverted view turn (`race::frame_draw`); now each view's
    /// (`scene::view_matrices::ViewMatrices::view_turn_copy`); unused.
    pub view_turn_copy: [PlainFloat; 16],
    /// The view's place matrix: the copy of `placed_matrix` (the view's place negated) made in
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
    /// A block whose first 0x40 bytes receive a copy of the made object matrix
    /// (`race::frame_draw`); now each view's (`scene::view_matrices`); unused.
    pub object_matrix_slots: [PlainDword; 32],
    /// The view's matrix, place (the last row of its 4 x 4 floats) (`effects::view_motion`, `race::frame_views`).
    pub view_place: [PlainFloat; 16],
    /// Unused; it only keeps the layout.
    pub unused_object_matrix: PlainDword,
    /// The vector reflected (`scene::reflect_vector`); now each view's
    /// (`scene::view_matrices`); unused.
    pub reflect_vector: [PlainFloat; 3],
    /// The normal (`scene::reflect_vector`); now each view's (`scene::view_matrices`); unused.
    pub reflect_normal: [PlainFloat; 3],
    /// The object's turn matrix (`physics::object_turn_matrix`, `physics::object_hits_car`,
    /// `race::frame_draw`).
    pub object_turn_matrix: [PlainFloat; 16],
    /// 1092-byte item, only the first 0x40 used as a matrix in place (`race::frame_draw`); now each
    /// view's (`scene::view_matrices`); unused.
    pub matrix_scratch: [PlainFloat; 273],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with its first values.
impl SceneMatrices {
    pub const fn new() -> Self {
        Self {
            view_work: {
                let mut a: [PlainFloat; 32] = ZeroCells::ZERO;
                a[0] = PlainFloat::new(-1.0);
                a[5] = PlainFloat::new(1.0);
                a[10] = PlainFloat::new(-1.0);
                a[15] = PlainFloat::new(1.0);
                a[16] = PlainFloat::new(-1.0);
                a[21] = PlainFloat::new(1.0);
                a[26] = PlainFloat::new(1.0);
                a[31] = PlainFloat::new(1.0);
                a
            },
            identity_matrix: {
                let mut a: [PlainFloat; 16] = ZeroCells::ZERO;
                a[0] = PlainFloat::new(1.0);
                a[5] = PlainFloat::new(1.0);
                a[10] = PlainFloat::new(1.0);
                a[15] = PlainFloat::new(1.0);
                a
            },
            turn_x_matrix: {
                let mut a: [PlainFloat; 16] = ZeroCells::ZERO;
                a[0] = PlainFloat::new(1.0);
                a[15] = PlainFloat::new(1.0);
                a
            },
            turn_y_matrix: {
                let mut a: [PlainFloat; 16] = ZeroCells::ZERO;
                a[5] = PlainFloat::new(1.0);
                a[15] = PlainFloat::new(1.0);
                a
            },
            turn_z_matrix: {
                let mut a: [PlainFloat; 16] = ZeroCells::ZERO;
                a[10] = PlainFloat::new(1.0);
                a[15] = PlainFloat::new(1.0);
                a
            },
            placed_matrix: {
                let mut a: [PlainFloat; 16] = ZeroCells::ZERO;
                a[0] = PlainFloat::new(1.0);
                a[5] = PlainFloat::new(1.0);
                a[10] = PlainFloat::new(1.0);
                a[15] = PlainFloat::new(1.0);
                a
            },
            ..ZeroCells::ZERO
        }
    }
}
