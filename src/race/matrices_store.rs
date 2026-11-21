//! The scene's matrices in the race block: the physics' and the car parts' work matrices as a plain
//! record of values (`SceneMatrices`), and the views' matrices kept for what is drawn or heard
//! after them (`ViewKept`, plain cells). The matrix functions take and return values
//! (`scene::matrix::Matrix`); the callers store the results here. The identity and the mirror
//! work matrix are functions, and the turn and placed matrices, whose unset floats never change,
//! are the bases the turns start from (`scene::matrix::TURN_X_BASE` ..).

use crate::memory::session::{DrawPlain, PlainRead};
use crate::scene::matrix::Matrix;

/// The matrices the physics and the car's parts work in, as plain values. A plain record
/// (`session::DrawPlain`): written on the game's thread between frames, and read by the drawing
/// parts as a copy made when a drawing scope opens (the fourth columns of the turned, lit and
/// made-object matrices, `scene::view_matrices`).
#[derive(Clone, Copy)]
pub struct SceneMatrices {
    /// The car body's turn matrix copied from the car (`race::car_parts`).
    pub body_turn_matrix: Matrix,
    /// `drive::set_listener` copies the made object matrix into it; `effects::draw_shadow` turns
    /// the shadow's corners by it.
    pub listener_matrix: Matrix,
    /// The turned matrix (`physics::move_slot`, `physics::spin_matrix`, `race::car_parts`).
    pub turned_matrix: Matrix,
    /// The lit matrix (`race::car_parts`, `physics::object_turn_matrix`).
    pub lit_matrix: Matrix,
    /// The matrix made for an object (`drive::set_listener`).
    pub made_object_matrix: Matrix,
    /// The object's turn matrix (`physics::object_turn_matrix`, `physics::object_hits_car`).
    pub object_turn_matrix: Matrix,
}

impl SceneMatrices {
    pub const fn new() -> Self {
        Self {
            body_turn_matrix: [0.0; 16],
            listener_matrix: [0.0; 16],
            turned_matrix: [0.0; 16],
            lit_matrix: [0.0; 16],
            made_object_matrix: [0.0; 16],
            object_turn_matrix: [0.0; 16],
        }
    }
}

/// The matrices as one reader finds them: the game's thread's own (read and written through the
/// store, short borrows), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum SceneMatricesRef<'a> {
    Store(&'a DrawPlain<SceneMatrices>),
    Copy(&'a SceneMatrices),
}

impl SceneMatricesRef<'_> {
    /// The record as it stands.
    pub fn get(self) -> SceneMatrices {
        match self {
            Self::Store(store) => *store.borrow(),
            Self::Copy(record) => *record,
        }
    }
}

impl<'a> From<&'a DrawPlain<SceneMatrices>> for SceneMatricesRef<'a> {
    fn from(store: &'a DrawPlain<SceneMatrices>) -> Self {
        Self::Store(store)
    }
}

impl<'a> From<&'a PlainRead<SceneMatrices>> for SceneMatricesRef<'a> {
    fn from(copy: &'a PlainRead<SceneMatrices>) -> Self {
        Self::Copy(copy)
    }
}

/// The view's matrices kept after the views are drawn, read outside the drawing (the effects, the
/// listener, the heard turn): a plain record (`session::DrawPlain`) of values, written on the
/// game's thread between frames and read as a copy.
#[derive(Clone, Copy)]
pub struct ViewKept {
    /// The view's place matrix: the copy of the placed matrix (the view's place negated) made in
    /// `race::frame_draw::view_place_tail`.
    pub view_place_matrix: Matrix,
    /// The view turned back (`scene::view_matrix`, `scene::reflect_vector`).
    pub view_back_matrix: Matrix,
    /// The view's turn (`drive::set_listener`, `physics::car_sounds`, `race::frame_draw`).
    pub view_turn_matrix: Matrix,
    /// The view matrix and its elements, used in place (`scene::project_point`,
    /// `effects::draw_particle`, `race::skidmarks`).
    pub view_matrix: Matrix,
    /// The view's matrix, place (the last row of its 4 x 4 floats) (`effects::view_motion`, `race::frame_views`).
    pub view_place: Matrix,
}

impl ViewKept {
    pub const fn new() -> Self {
        Self {
            view_place_matrix: [0.0; 16],
            view_back_matrix: [0.0; 16],
            view_turn_matrix: [0.0; 16],
            view_matrix: [0.0; 16],
            view_place: [0.0; 16],
        }
    }
}

/// The 4x4 identity.
pub fn identity_matrix() -> Matrix {
    let mut a: Matrix = [0.0; 16];
    a[0] = 1.0;
    a[5] = 1.0;
    a[10] = 1.0;
    a[15] = 1.0;
    a
}

/// The constant matrix the frame draw turns a mirrored view's car by: the diagonal (-1, 1, -1, 1).
pub fn mirror_work_matrix() -> Matrix {
    let mut a: Matrix = [0.0; 16];
    a[0] = -1.0;
    a[5] = 1.0;
    a[10] = -1.0;
    a[15] = 1.0;
    a
}
