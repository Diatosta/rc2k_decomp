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
    /// The inverted view turn kept for the mirror (`SceneMatrices.view_turn_copy[0]`), each view's own.
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
/// one after another. A drawing part other than the first (`workers::draw_parts`) holds from
/// and puts back into its own columns (`Columns`), started as the list began; what it changed
/// goes into the store in the parts' order (`merge_columns`).
pub struct Held<'a> {
    view: &'a ViewMatrices,
    store: &'a SceneMatrices,
}

/// The fourth columns of the turned, lit and made-object matrices, in that order.
#[derive(Clone, Copy, PartialEq)]
pub struct Columns([u32; 12]);

const CELLS: [usize; 4] = [3, 7, 11, 15];

fn held(m: &SceneMatrices) -> [&[Float; 16]; 3] {
    [&m.turned_matrix, &m.lit_matrix, &m.made_object_matrix]
}

fn view_held(m: &ViewMatrices) -> [&[Float; 16]; 3] {
    [&m.turned_matrix, &m.lit_matrix, &m.made_object_matrix]
}

fn read_columns(from: [&[Float; 16]; 3]) -> Columns {
    Columns(std::array::from_fn(|n| from[n / 4][CELLS[n % 4]].bits()))
}

fn write_columns(c: &Columns, to: [&[Float; 16]; 3]) {
    for (n, &value) in c.0.iter().enumerate() {
        to[n / 4][CELLS[n % 4]].set_bits(value);
    }
}

thread_local! {
    /// The running part's columns, if the thread is drawing a part other than the first.
    static OWN: std::cell::Cell<Option<Columns>> = const { std::cell::Cell::new(None) };
}

/// Matrix `m` (0 turned, 1 lit, 2 made-object) as this thread's part has it: the store's
/// first three columns, the fourth the part's own (`OWN`) in a part other than the first.
pub fn current_matrix(store: &SceneMatrices, m: usize) -> [u32; 16] {
    let c = OWN.get();
    std::array::from_fn(|cell| match (c, CELLS.iter().position(|&k| k == cell)) {
        (Some(c), Some(row)) => c.0[m * 4 + row],
        _ => held(store)[m][cell].bits(),
    })
}

/// The store's columns now.
pub fn columns(store: &SceneMatrices) -> Columns {
    read_columns(held(store))
}

/// A part begins on this thread, its columns `start`.
pub fn begin_part(start: Columns) {
    OWN.set(Some(start));
}

/// The part on this thread is over: the columns it leaves.
pub fn end_part() -> Option<Columns> {
    OWN.take()
}

/// A part's columns `left` into the store where they differ from `start` (as the copies'
/// byte merge did).
pub fn merge_columns(store: &SceneMatrices, start: &Columns, left: &Columns) {
    let to = held(store);
    for (n, (&s, &l)) in start.0.iter().zip(&left.0).enumerate() {
        if s != l {
            to[n / 4][CELLS[n % 4]].set_bits(l);
        }
    }
}

impl ViewMatrices {
    /// The shared turned, lit and made-object matrices' fourth columns taken into the view's
    /// until the returned hold is dropped.
    pub fn hold<'a>(&'a self, store: &'a SceneMatrices) -> Held<'a> {
        let c = OWN.get().unwrap_or_else(|| columns(store));
        write_columns(&c, view_held(self));
        Held { view: self, store }
    }
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        let c = read_columns(view_held(self.view));
        if OWN.get().is_some() {
            OWN.set(Some(c));
        } else {
            write_columns(&c, held(self.store));
        }
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
