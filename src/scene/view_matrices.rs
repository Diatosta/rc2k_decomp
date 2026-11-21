//! A view's matrices and turning work: what placing and turning an object, a car or a face works
//! out in the view, one in each view's slot (`race::view_scratch`) and one for the front end.
//!
//! The scene matrices (`race::matrices_store`: the three turning matrices, the placed, turned, lit
//! and made-object matrices, the reflection's vectors, the object's turn) and the scene draw state
//! (`scene::draw_store`: the turnings' sines and cosines) are shared by every view in turn; each is
//! written before it is read within a view.
//!
//! Kept in the shared pieces (the light's direction is the view's `ObjectLight`'s), being the
//! view's parameters read outside the drawing (the effects, the listener, the heard turn) or kept
//! from view to view: the view matrix, the view turn and its copy, the view's place, the view back
//! matrix, the identity, the frame draw's two work matrices, and the listener's matrix.

use crate::race::matrices_store::{SceneMatrices, SceneMatricesRef};
use crate::race::plain_slot::{Merge, PlainSlot, merge_floats, merge_run, merge_value};
use crate::scene::matrix::{Matrix, PLACED_BASE, TURN_X_BASE, TURN_Y_BASE, TURN_Z_BASE};

/// A view's matrices and turning work, behind a `RefCell`: borrows are held for one statement.
pub type ViewMatrices = PlainSlot<MatrixValues>;

/// The values; the floats merge by their bits.
#[derive(Clone, Copy)]
pub struct MatrixValues {
    /// The model last set up for drawing (`scene::load_object`, `race::car_draw`).
    pub known_object: u32,
    /// Rotation about x (`scene::matrix`).
    pub turn_x_matrix: [f32; 16],
    /// Rotation about y.
    pub turn_y_matrix: [f32; 16],
    /// Rotation about z.
    pub turn_z_matrix: [f32; 16],
    /// A matrix with its place.
    pub placed_matrix: [f32; 16],
    /// The car body's turn matrix copied from the car (`race::car_parts`).
    pub body_turn_matrix: [f32; 16],
    /// The turned matrix.
    pub turned_matrix: [f32; 16],
    /// The lit matrix.
    pub lit_matrix: [f32; 16],
    /// The matrix made for an object.
    pub made_object_matrix: [f32; 16],
    /// The made object matrix's bits as the sky's draw left them, the sky's description
    /// (`race::frame_draw`).
    pub object_matrix_slots: [u32; 16],
    /// The vector reflected (`scene::reflect_vector`).
    pub reflect_vector: [f32; 3],
    /// The normal (`scene::reflect_vector`).
    pub reflect_normal: [f32; 3],
    /// The object's turn matrix (`race::frame_draw`).
    pub object_turn_matrix: [f32; 16],
    /// The matrix the view's place is moved into (`race::frame_draw`).
    pub matrix_scratch: [f32; 16],
    /// The work vector the reflection and the faces' facing work in.
    pub work_vector: [u32; 4],
    /// The x turning's sine and cosine (`scene::set_turn_x`).
    pub turn_x_work: [f32; 2],
    /// The y turning's.
    pub turn_y_work: [f32; 2],
    /// The z turning's.
    pub turn_z_work: [f32; 2],
    /// The view matrix, each view's own (written by `race::camera_view`; the shared one is kept
    /// for the effects drawn after the views).
    pub view_matrix: [f32; 16],
    /// The view's turn, each view's own.
    pub view_turn_matrix: [f32; 16],
    /// The view turned back, each view's own.
    pub view_back_matrix: [f32; 16],
    /// The view's place matrix, each view's own.
    pub view_place_matrix: [f32; 16],
    /// The inverted view turn kept for the mirror, each view's own.
    pub view_turn_copy: [f32; 16],
}

impl Merge for MatrixValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_value(
            &mut self.known_object,
            &now.known_object,
            &start.known_object,
        );
        merge_floats(
            &mut self.turn_x_matrix,
            &now.turn_x_matrix,
            &start.turn_x_matrix,
        );
        merge_floats(
            &mut self.turn_y_matrix,
            &now.turn_y_matrix,
            &start.turn_y_matrix,
        );
        merge_floats(
            &mut self.turn_z_matrix,
            &now.turn_z_matrix,
            &start.turn_z_matrix,
        );
        merge_floats(
            &mut self.placed_matrix,
            &now.placed_matrix,
            &start.placed_matrix,
        );
        merge_floats(
            &mut self.body_turn_matrix,
            &now.body_turn_matrix,
            &start.body_turn_matrix,
        );
        merge_floats(
            &mut self.turned_matrix,
            &now.turned_matrix,
            &start.turned_matrix,
        );
        merge_floats(&mut self.lit_matrix, &now.lit_matrix, &start.lit_matrix);
        merge_floats(
            &mut self.made_object_matrix,
            &now.made_object_matrix,
            &start.made_object_matrix,
        );
        merge_run(
            &mut self.object_matrix_slots,
            &now.object_matrix_slots,
            &start.object_matrix_slots,
        );
        merge_floats(
            &mut self.reflect_vector,
            &now.reflect_vector,
            &start.reflect_vector,
        );
        merge_floats(
            &mut self.reflect_normal,
            &now.reflect_normal,
            &start.reflect_normal,
        );
        merge_floats(
            &mut self.object_turn_matrix,
            &now.object_turn_matrix,
            &start.object_turn_matrix,
        );
        merge_floats(
            &mut self.matrix_scratch,
            &now.matrix_scratch,
            &start.matrix_scratch,
        );
        merge_run(&mut self.work_vector, &now.work_vector, &start.work_vector);
        merge_floats(&mut self.turn_x_work, &now.turn_x_work, &start.turn_x_work);
        merge_floats(&mut self.turn_y_work, &now.turn_y_work, &start.turn_y_work);
        merge_floats(&mut self.turn_z_work, &now.turn_z_work, &start.turn_z_work);
        merge_floats(&mut self.view_matrix, &now.view_matrix, &start.view_matrix);
        merge_floats(
            &mut self.view_turn_matrix,
            &now.view_turn_matrix,
            &start.view_turn_matrix,
        );
        merge_floats(
            &mut self.view_back_matrix,
            &now.view_back_matrix,
            &start.view_back_matrix,
        );
        merge_floats(
            &mut self.view_place_matrix,
            &now.view_place_matrix,
            &start.view_place_matrix,
        );
        merge_floats(
            &mut self.view_turn_copy,
            &now.view_turn_copy,
            &start.view_turn_copy,
        );
    }
}

/// The fourth columns (cells 3, 7, 11, 15) of the turned, lit and made-object matrices, held from
/// the shared store while a drawing runs (`ViewMatrices::hold`) and put back when it ends: a
/// hand-off both ways. A product (`scene::multiply_matrices`) writes three columns only, so the
/// fourth keeps what the last whole copy or scaling left (the size, in the last cell, after an
/// object's scaling). The physics copies all 16 cells on (`physics::spin_matrix` into the crash
/// matrix, `physics::move_slot` into a slot's, `drive::set_listener` into the listener's), and the
/// descriptions the drawing reads (0x3C bytes) take what the physics left. The other columns are
/// written before they are read on both sides. Exact while the views are drawn one after another. A
/// drawing part other than the first (`scene::workers::draw_parts`) holds from and puts back into
/// its own columns (`Columns`), started as the list began; what it changed goes into the store in
/// the parts' order (`merge_columns`).
pub struct Held<'a> {
    view: &'a ViewMatrices,
    store: SceneMatricesRef<'a>,
    part: &'a crate::scene::scratch_store::PartScratch,
}

/// The fourth columns of the turned, lit and made-object matrices, in that order, as their
/// bits: a part's are compared with the start's for change and put back as they were (a NaN
/// or a negative zero kept).
#[derive(Clone, Copy, PartialEq)]
pub struct Columns([u32; 12]);

const CELLS: [usize; 4] = [3, 7, 11, 15];

fn held(m: &SceneMatrices) -> [&Matrix; 3] {
    [&m.turned_matrix, &m.lit_matrix, &m.made_object_matrix]
}

fn held_mut(m: &mut SceneMatrices) -> [&mut Matrix; 3] {
    [
        &mut m.turned_matrix,
        &mut m.lit_matrix,
        &mut m.made_object_matrix,
    ]
}

fn view_held(m: &MatrixValues) -> [&Matrix; 3] {
    [&m.turned_matrix, &m.lit_matrix, &m.made_object_matrix]
}

fn view_held_mut(m: &mut MatrixValues) -> [&mut Matrix; 3] {
    [
        &mut m.turned_matrix,
        &mut m.lit_matrix,
        &mut m.made_object_matrix,
    ]
}

fn plain_columns(from: [&Matrix; 3]) -> Columns {
    Columns(std::array::from_fn(|n| from[n / 4][CELLS[n % 4]].to_bits()))
}

fn write_columns(c: &Columns, to: [&mut Matrix; 3]) {
    for (n, &value) in c.0.iter().enumerate() {
        to[n / 4][CELLS[n % 4]] = f32::from_bits(value);
    }
}

/// A drawing part's own columns (`scene::scratch_store::PartScratch::columns`): set while a part other
/// than the first is drawn, none otherwise. A plain cell: only the thread that draws the part
/// reaches it (`memory::part_cell::PartCell`).
pub struct OwnColumns(std::cell::Cell<Option<Columns>>);

impl OwnColumns {
    pub const fn new() -> Self {
        Self(std::cell::Cell::new(None))
    }

    fn get(&self) -> Option<Columns> {
        self.0.get()
    }

    fn set(&self, c: Option<Columns>) -> Option<Columns> {
        self.0.replace(c)
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
    part: &crate::scene::scratch_store::PartScratch,
    m: usize,
) -> [u32; 16] {
    let c = part.columns.get();
    std::array::from_fn(|cell| match (c, CELLS.iter().position(|&k| k == cell)) {
        (Some(c), Some(row)) => c.0[m * 4 + row],
        _ => held(store)[m][cell].to_bits(),
    })
}

/// The store's columns now.
pub fn columns(store: &SceneMatrices) -> Columns {
    plain_columns(held(store))
}

/// Part `part` (not the first) begins, its columns `start`.
pub fn begin_part(part: &crate::scene::scratch_store::PartScratch, start: Columns) {
    part.columns.set(Some(start));
}

/// The columns `part` holds of its own, if any (a part being drawn).
pub fn part_columns(part: &crate::scene::scratch_store::PartScratch) -> Option<Columns> {
    part.columns.get()
}

/// `part`'s own columns made `columns` (a piece drawn aside starts from its part's,
/// `scene::workers::draw_aside`).
pub fn set_part_columns(part: &crate::scene::scratch_store::PartScratch, columns: Option<Columns>) {
    part.columns.set(columns);
}

/// Part `part` is over: the columns it leaves.
pub fn end_part(part: &crate::scene::scratch_store::PartScratch) -> Option<Columns> {
    part.columns.set(None)
}

/// A part's columns `left` into the store where they differ from `start`.
pub fn merge_columns(
    store: &crate::memory::session::DrawPlain<SceneMatrices>,
    start: &Columns,
    left: &Columns,
) {
    let mut kept = store.borrow_mut();
    let to = held_mut(&mut kept);
    for (n, (&s, &l)) in start.0.iter().zip(&left.0).enumerate() {
        if s != l {
            to[n / 4][CELLS[n % 4]] = f32::from_bits(l);
        }
    }
}

/// The shared turned, lit and made-object matrices' fourth columns (`part`'s own in a
/// part other than the first) taken into `view`'s until the returned hold is dropped.
pub fn hold<'a>(
    view: &'a ViewMatrices,
    store: SceneMatricesRef<'a>,
    part: &'a crate::scene::scratch_store::PartScratch,
) -> Held<'a> {
    let c = part.columns.get().unwrap_or_else(|| columns(&store.get()));
    write_columns(&c, view_held_mut(&mut view.borrow_mut()));
    Held { view, store, part }
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        let c = plain_columns(view_held(&self.view.borrow()));
        if self.part.columns.get().is_some() {
            self.part.columns.set(Some(c));
        } else {
            // Only a hold outside a drawing scope (the front end's) reaches the store itself: a
            // drawing part's copy is never written (every part holds columns of its own).
            match self.store {
                SceneMatricesRef::Store(store) => {
                    let mut kept = store.borrow_mut();
                    let to = held_mut(&mut kept);
                    for (n, &value) in c.0.iter().enumerate() {
                        to[n / 4][CELLS[n % 4]] = f32::from_bits(value);
                    }
                }
                SceneMatricesRef::Copy(_) => {
                    debug_assert!(false, "a drawing part held without columns of its own");
                }
            }
        }
    }
}

impl MatrixValues {
    /// With the image's first values (an identity's floats in the turning matrices).
    pub const fn new() -> Self {
        Self {
            known_object: 0,
            turn_x_matrix: TURN_X_BASE,
            turn_y_matrix: TURN_Y_BASE,
            turn_z_matrix: TURN_Z_BASE,
            placed_matrix: PLACED_BASE,
            body_turn_matrix: [0.0; 16],
            turned_matrix: [0.0; 16],
            lit_matrix: [0.0; 16],
            made_object_matrix: [0.0; 16],
            object_matrix_slots: [0; 16],
            reflect_vector: [0.0; 3],
            reflect_normal: [0.0; 3],
            object_turn_matrix: [0.0; 16],
            matrix_scratch: [0.0; 16],
            work_vector: [0; 4],
            turn_x_work: [0.0; 2],
            turn_y_work: [0.0; 2],
            turn_z_work: [0.0; 2],
            view_matrix: [0.0; 16],
            view_turn_matrix: [0.0; 16],
            view_back_matrix: [0.0; 16],
            view_place_matrix: [0.0; 16],
            view_turn_copy: [0.0; 16],
        }
    }
}
