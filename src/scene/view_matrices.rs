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
use crate::session::{Cell32, PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewMatrices {
    /// The model last set up for drawing (`scene::load_object`, `race::car_draw`).
    pub known_object: PlainDword,
    /// Rotation about x (`scene::matrix`).
    pub turn_x_matrix: [PlainFloat; 16],
    /// Rotation about y.
    pub turn_y_matrix: [PlainFloat; 16],
    /// Rotation about z.
    pub turn_z_matrix: [PlainFloat; 16],
    /// A matrix with its place.
    pub placed_matrix: [PlainFloat; 16],
    /// The car body's turn matrix copied from the car (`race::car_parts`).
    pub body_turn_matrix: [PlainFloat; 16],
    /// The turned matrix.
    pub turned_matrix: [PlainFloat; 16],
    /// The lit matrix.
    pub lit_matrix: [PlainFloat; 16],
    /// The matrix made for an object.
    pub made_object_matrix: [PlainFloat; 16],
    /// A block whose first 0x40 bytes receive a copy of the made object matrix, drawn
    /// described from there (`race::frame_draw`).
    pub object_matrix_slots: [PlainDword; 32],
    /// The vector reflected (`scene::reflect_vector`).
    pub reflect_vector: [PlainFloat; 3],
    /// The normal (`scene::reflect_vector`).
    pub reflect_normal: [PlainFloat; 3],
    /// The object's turn matrix (`race::frame_draw`).
    pub object_turn_matrix: [PlainFloat; 16],
    /// The matrix the view's place is moved into (`race::frame_draw`).
    pub matrix_scratch: [PlainFloat; 16],
    /// The work vector (`g_workVector`) the reflection and the faces' facing work in.
    pub work_vector: [PlainDword; 4],
    /// The x turning's sine and cosine (`scene::set_turn_x`).
    pub turn_x_work: [PlainFloat; 2],
    /// The y turning's.
    pub turn_y_work: [PlainFloat; 2],
    /// The z turning's.
    pub turn_z_work: [PlainFloat; 2],
    /// The view matrix (`g_viewMatrix`, 0x5EA2CC), each view's own (written by
    /// `race::camera_view`; the shared one is kept for the effects drawn after the views).
    pub view_matrix: [PlainFloat; 16],
    /// The view's turn (`g_viewTurnMatrix`, 0x5EA24C), each view's own.
    pub view_turn_matrix: [PlainFloat; 16],
    /// The view turned back (`g_viewBackMatrix`, 0x5EA20C), each view's own.
    pub view_back_matrix: [PlainFloat; 16],
    /// The view's place matrix (0x5EA1CC), each view's own.
    pub view_place_matrix: [PlainFloat; 16],
    /// The inverted view turn kept for the mirror (`SceneMatrices.view_turn_copy[0]`), each view's own.
    pub view_turn_copy: [PlainFloat; 16],
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
    part: &'a crate::workers::PartScratch,
}

/// The fourth columns of the turned, lit and made-object matrices, in that order, as their
/// bits: a part's are compared with the start's for change and put back as they were (a NaN
/// or a negative zero kept).
#[derive(Clone, Copy, PartialEq)]
pub struct Columns([u32; 12]);

const CELLS: [usize; 4] = [3, 7, 11, 15];

fn held(m: &SceneMatrices) -> [&[PlainFloat; 16]; 3] {
    [&m.turned_matrix, &m.lit_matrix, &m.made_object_matrix]
}

fn view_held(m: &ViewMatrices) -> [&[PlainFloat; 16]; 3] {
    [&m.turned_matrix, &m.lit_matrix, &m.made_object_matrix]
}

fn read_columns<C: Cell32>(from: [&[C; 16]; 3]) -> Columns {
    Columns(std::array::from_fn(|n| from[n / 4][CELLS[n % 4]].bits()))
}

fn write_columns<C: Cell32>(c: &Columns, to: [&[C; 16]; 3]) {
    for (n, &value) in c.0.iter().enumerate() {
        to[n / 4][CELLS[n % 4]].set_bits(value);
    }
}

/// A drawing part's own columns (`workers::PartScratch::columns`): set while a part other
/// than the first is drawn, none otherwise.
pub struct OwnColumns(std::sync::Mutex<Option<Columns>>);

impl OwnColumns {
    pub const fn new() -> Self {
        Self(std::sync::Mutex::new(None))
    }

    fn get(&self) -> Option<Columns> {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn set(&self, c: Option<Columns>) -> Option<Columns> {
        std::mem::replace(
            &mut *self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            c,
        )
    }
}

impl Default for OwnColumns {
    fn default() -> Self {
        Self::new()
    }
}

/// Matrix `m` (0 turned, 1 lit, 2 made-object) as `part` has it: the store's first three
/// columns, the fourth the part's own in a part other than the first.
pub fn current_matrix(
    store: &SceneMatrices,
    part: &crate::workers::PartScratch,
    m: usize,
) -> [u32; 16] {
    let c = part.columns.get();
    std::array::from_fn(|cell| match (c, CELLS.iter().position(|&k| k == cell)) {
        (Some(c), Some(row)) => c.0[m * 4 + row],
        _ => held(store)[m][cell].bits(),
    })
}

/// The store's columns now.
pub fn columns(store: &SceneMatrices) -> Columns {
    read_columns(held(store))
}

/// Part `part` (not the first) begins, its columns `start`.
pub fn begin_part(part: &crate::workers::PartScratch, start: Columns) {
    part.columns.set(Some(start));
}

/// Part `part` is over: the columns it leaves.
pub fn end_part(part: &crate::workers::PartScratch) -> Option<Columns> {
    part.columns.set(None)
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
    /// The shared turned, lit and made-object matrices' fourth columns (`part`'s own in a
    /// part other than the first) taken into the view's until the returned hold is dropped.
    pub fn hold<'a>(
        &'a self,
        store: &'a SceneMatrices,
        part: &'a crate::workers::PartScratch,
    ) -> Held<'a> {
        let c = part.columns.get().unwrap_or_else(|| columns(store));
        write_columns(&c, view_held(self));
        Held {
            view: self,
            store,
            part,
        }
    }
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        let c = read_columns(view_held(self.view));
        if self.part.columns.get().is_some() {
            self.part.columns.set(Some(c));
        } else {
            write_columns(&c, held(self.store));
        }
    }
}

impl ViewMatrices {
    /// With the image's first values (an identity's floats in the turning matrices).
    pub const fn new() -> Self {
        /// A matrix of zeros but for the ones at `ones`.
        const fn ones_at<const N: usize>(ones: [usize; N]) -> [PlainFloat; 16] {
            let mut a = [const { PlainFloat::new(0.0) }; 16];
            let mut k = 0;
            while k < N {
                a[ones[k]] = PlainFloat::new(1.0);
                k += 1;
            }
            a
        }
        Self {
            known_object: PlainDword::new(0),
            turn_x_matrix: ones_at([0, 15]),
            turn_y_matrix: ones_at([5, 15]),
            turn_z_matrix: ones_at([10, 15]),
            placed_matrix: ones_at([0, 5, 10, 15]),
            body_turn_matrix: [const { PlainFloat::new(0.0) }; 16],
            turned_matrix: [const { PlainFloat::new(0.0) }; 16],
            lit_matrix: [const { PlainFloat::new(0.0) }; 16],
            made_object_matrix: [const { PlainFloat::new(0.0) }; 16],
            object_matrix_slots: [const { PlainDword::new(0) }; 32],
            reflect_vector: [const { PlainFloat::new(0.0) }; 3],
            reflect_normal: [const { PlainFloat::new(0.0) }; 3],
            object_turn_matrix: [const { PlainFloat::new(0.0) }; 16],
            matrix_scratch: [const { PlainFloat::new(0.0) }; 16],
            work_vector: [const { PlainDword::new(0) }; 4],
            turn_x_work: [const { PlainFloat::new(0.0) }; 2],
            turn_y_work: [const { PlainFloat::new(0.0) }; 2],
            turn_z_work: [const { PlainFloat::new(0.0) }; 2],
            view_matrix: [const { PlainFloat::new(0.0) }; 16],
            view_turn_matrix: [const { PlainFloat::new(0.0) }; 16],
            view_back_matrix: [const { PlainFloat::new(0.0) }; 16],
            view_place_matrix: [const { PlainFloat::new(0.0) }; 16],
            view_turn_copy: [const { PlainFloat::new(0.0) }; 16],
        }
    }
}
