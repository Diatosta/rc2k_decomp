//! A view's matrices and turning work (frame plan F4): what placing and turning an object, a
//! car or a face works out in the view, one in each view's slot (`race::view_scratch`) and
//! one for the front end.
//!
//! The original kept it in the scene matrices (`race::matrices_store`: the three turning
//! matrices, the placed, turned, lit and made-object matrices, the reflection's vectors, the
//! object's turn) and in the scene draw state (`scene::draw_store`: the turnings' sines and
//! cosines), shared by every view in turn. Each is written before it
//! is read within a view.
//!
//! Kept in the shared pieces (the light's direction is the view's `ObjectLight`'s since F6),
//! being the view's parameters read outside the drawing (the
//! effects, the listener, the heard turn) or kept from view to view: the view matrix, the
//! view turn and its copy, the view's place, the view back matrix, the identity, the frame
//! draw's two work matrices, and the listener's matrix.

use crate::race::matrices_store::SceneMatrices;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewMatrices {
    /// The model last set up for drawing (`scene::load_object`, `race::car_draw`).
    pub known_object: Dword,
    /// Rotation about x (`scene::matrix`).
    pub turn_x_matrix: [Float; 16],
    /// Rotation about y.
    pub turn_y_matrix: [Float; 16],
    /// Rotation about z.
    pub turn_z_matrix: [Float; 16],
    /// A matrix with its place.
    pub placed_matrix: [Float; 16],
    /// The car body's turn matrix copied from the car (`race::car_parts`).
    pub body_turn_matrix: [Float; 16],
    /// The turned matrix.
    pub turned_matrix: [Float; 16],
    /// The lit matrix.
    pub lit_matrix: [Float; 16],
    /// The matrix made for an object.
    pub made_object_matrix: [Float; 16],
    /// A block whose first 0x40 bytes receive a copy of the made object matrix, drawn
    /// described from there (`race::frame_draw`).
    pub object_matrix_slots: [Dword; 32],
    /// The vector reflected (`scene::reflect_vector`).
    pub reflect_vector: [Float; 3],
    /// The normal (`scene::reflect_vector`).
    pub reflect_normal: [Float; 3],
    /// The object's turn matrix (`race::frame_draw`).
    pub object_turn_matrix: [Float; 16],
    /// The matrix the view's place is moved into (`race::frame_draw`).
    pub matrix_scratch: [Float; 16],
    /// The work vector (`g_workVector`) the reflection and the faces' facing work in.
    pub work_vector: [Dword; 4],
    /// The x turning's sine and cosine (`scene::set_turn_x`).
    pub turn_x_work: [Float; 2],
    /// The y turning's.
    pub turn_y_work: [Float; 2],
    /// The z turning's.
    pub turn_z_work: [Float; 2],
    /// The view matrix (`g_viewMatrix`, 0x5EA2CC), each view's own (written by
    /// `race::camera_view`; the shared one is kept for the effects drawn after the views).
    pub view_matrix: [Float; 16],
    /// The view's turn (`g_viewTurnMatrix`, 0x5EA24C), each view's own.
    pub view_turn_matrix: [Float; 16],
    /// The view turned back (`g_viewBackMatrix`, 0x5EA20C), each view's own.
    pub view_back_matrix: [Float; 16],
    /// The view's place matrix (0x5EA1CC), each view's own.
    pub view_place_matrix: [Float; 16],
    /// The inverted view turn kept for the mirror (`flt_5EA18C`), each view's own.
    pub view_turn_copy: [Float; 16],
}

/// The fourth columns (cells 3, 7, 11, 15) of the turned, lit and made-object matrices, held
/// from the shared store while a drawing runs (`ViewMatrices::hold`) and put back when it
/// ends: a hand-off both ways. A product (`scene::multiply_matrices`) writes three columns only, so the fourth
/// keeps what the last whole copy or scaling left (the size in +0x3C after an object's
/// scaling). The physics copies all 16 cells on (`physics::spin_matrix` into the crash
/// matrix, `physics::move_slot` into a slot's, `drive::set_listener` into the listener's),
/// and the descriptions the drawing reads (0x3C bytes) take what the physics left. The other
/// columns are written before they are read on both sides. Exact while the views are drawn
/// one after another.
pub struct Held<'a> {
    view: &'a ViewMatrices,
    store: &'a SceneMatrices,
}

fn copy_matrices(from: [&[Float; 16]; 3], to: [&[Float; 16]; 3]) {
    for (from, to) in from.into_iter().zip(to) {
        for cell in [3, 7, 11, 15] {
            to[cell].set_bits(from[cell].bits());
        }
    }
}

impl ViewMatrices {
    /// The shared turned, lit and made-object matrices' fourth columns taken into the view's
    /// until the returned hold is dropped.
    pub fn hold<'a>(&'a self, store: &'a SceneMatrices) -> Held<'a> {
        copy_matrices(
            [
                &store.turned_matrix,
                &store.lit_matrix,
                &store.made_object_matrix,
            ],
            [
                &self.turned_matrix,
                &self.lit_matrix,
                &self.made_object_matrix,
            ],
        );
        Held { view: self, store }
    }
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        let (v, s) = (self.view, self.store);
        copy_matrices(
            [&v.turned_matrix, &v.lit_matrix, &v.made_object_matrix],
            [&s.turned_matrix, &s.lit_matrix, &s.made_object_matrix],
        );
    }
}

impl ViewMatrices {
    /// With the image's first values (an identity's floats in the turning matrices).
    pub const fn new() -> Self {
        let s = SceneMatrices::new();
        let SceneMatrices {
            known_object,
            turn_x_matrix,
            turn_y_matrix,
            turn_z_matrix,
            placed_matrix,
            body_turn_matrix,
            turned_matrix,
            lit_matrix,
            made_object_matrix,
            object_matrix_slots,
            reflect_vector,
            reflect_normal,
            object_turn_matrix,
            matrix_scratch,
            view_matrix,
            view_turn_matrix,
            view_back_matrix,
            view_place_matrix,
            view_turn_copy,
            ..
        } = s;
        let [
            m0,
            m1,
            m2,
            m3,
            m4,
            m5,
            m6,
            m7,
            m8,
            m9,
            m10,
            m11,
            m12,
            m13,
            m14,
            m15,
            ..,
        ] = matrix_scratch;
        Self {
            known_object,
            turn_x_matrix,
            turn_y_matrix,
            turn_z_matrix,
            placed_matrix,
            body_turn_matrix,
            turned_matrix,
            lit_matrix,
            made_object_matrix,
            object_matrix_slots,
            reflect_vector,
            reflect_normal,
            object_turn_matrix,
            matrix_scratch: [
                m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15,
            ],
            work_vector: [const { Dword::new(0) }; 4],
            turn_x_work: [const { Float::from_bits(0) }; 2],
            turn_y_work: [const { Float::from_bits(0) }; 2],
            turn_z_work: [const { Float::from_bits(0) }; 2],
            view_matrix,
            view_turn_matrix,
            view_back_matrix,
            view_place_matrix,
            view_turn_copy,
        }
    }
}
