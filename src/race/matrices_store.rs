//! The scene's matrices in the race block, 0x5E9EC8 .. 0x5EA8C8: the identity, the three turn matrices, the placed / turned / lit matrices, the view matrices, the object matrices and a scratch area.
//! Most are 4x4 float matrices (64 bytes) that Rust reaches in place by address (`ga(..)`) and passes to matrix functions; the generated struct overlays the same bytes.
//! 0x5EA484 is IDA's one 1092-byte item, taken whole; only its first 64 bytes are used.
//! Plain cells: a drawing store (`session::DrawShared`), written on the game's thread between
//! frames and read by the drawing parts through their `workers::DrawReads`.
//! Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SceneMatrices {
    /// 0x5E9EC8: `dword_5E9EC8`: the model last set up for drawing (`scene::load_object`, `race::car_draw`).
    pub known_object: PlainDword,
    /// 0x5E9ECC: `flt_5E9ECC`: two matrices of the frame draw, used in place as scratch (`race::frame_draw`).
    pub view_work: [PlainFloat; 32],
    /// 0x5E9F4C: `g_identityMatrix`: the 4x4 identity (`scene::view_matrix`, `race::frame_draw`).
    pub identity_matrix: [PlainFloat; 16],
    /// 0x5E9F8C: `g_turnXMatrix`: rotation about x (`scene::matrix`, `physics::object_turn_matrix`).
    pub turn_x_matrix: [PlainFloat; 16],
    /// 0x5E9FCC: `g_turnYMatrix`: rotation about y (`scene::matrix`, `race::car_parts`).
    pub turn_y_matrix: [PlainFloat; 16],
    /// 0x5EA00C: `g_turnZMatrix`: rotation about z (`scene::view_matrix`, `race::car_draw`).
    pub turn_z_matrix: [PlainFloat; 16],
    /// 0x5EA04C: `g_placedMatrix`: a matrix with its place (`scene::matrix`, `physics::car_collisions`).
    pub placed_matrix: [PlainFloat; 16],
    /// 0x5EA08C: `flt_5EA08C`: the car body's turn matrix copied from the car (`race::car_parts`, `race::car_draw`).
    pub body_turn_matrix: [PlainFloat; 16],
    /// 0x5EA0CC: `drive::set_listener` copies the made object matrix into it; `effects::draw_shadow` turns the shadow's corners by it.
    pub listener_matrix: [PlainFloat; 16],
    /// 0x5EA10C: `g_turnedMatrix`: the turned matrix (`race::frame_draw`, `physics::move_slot`, `scene::matrix`).
    pub turned_matrix: [PlainFloat; 16],
    /// 0x5EA14C: `g_litMatrix`: the lit matrix (`race::frame_draw`, `race::car_parts`).
    pub lit_matrix: [PlainFloat; 16],
    /// 0x5EA18C: `flt_5EA18C`: the inverted view turn (`race::frame_draw`); now each view's (`scene::view_matrices::ViewMatrices::view_turn_copy`); unused.
    pub view_turn_copy: [PlainFloat; 16],
    /// 0x5EA1CC: the view's place matrix: the copy of `placed_matrix` (the view's place negated) made in `race::frame_draw::view_place_tail`.
    pub view_place_matrix: [PlainFloat; 16],
    /// 0x5EA20C: `g_viewBackMatrix`: the view turned back (`scene::view_matrix`, `scene::reflect_vector`).
    pub view_back_matrix: [PlainFloat; 16],
    /// 0x5EA24C: `g_viewTurnMatrix`: the view's turn (`drive::set_listener`, `physics::car_sounds`, `race::frame_draw`).
    pub view_turn_matrix: [PlainFloat; 16],
    /// 0x5EA28C: no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_3c4: [PlainDword; 16],
    /// 0x5EA2CC: `g_viewMatrix` and its elements `flt_5EA2D0`..: the view matrix, used in place (`scene::project_point`, `effects::draw_particle`, `race::skidmarks`).
    pub view_matrix: [PlainFloat; 16],
    /// 0x5EA30C: `g_madeObjectMatrix`: the matrix made for an object (`scene::matrix`, `race::frame_draw`).
    pub made_object_matrix: [PlainFloat; 16],
    /// 0x5EA34C: `dword_5EA34C`: a block whose first 0x40 bytes receive a copy of the made object matrix (`race::frame_draw`); now each view's (`scene::view_matrices`); unused.
    pub object_matrix_slots: [PlainDword; 32],
    /// 0x5EA3CC: `flt_5EA3CC`: the view's matrix, place at +0x30 (`effects::view_motion`, `race::frame_views`).
    pub view_place: [PlainFloat; 16],
    /// 0x5EA40C: `g_objectMatrix`: pointer to the object matrix in use (`scene::place_object_points`).
    pub object_matrix_ptr: PlainDword,
    /// 0x5EA410: no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_548: [PlainDword; 6],
    /// 0x5EA428: `dword_5EA428`..: the vector reflected (`scene::reflect_vector`); now each view's (`scene::view_matrices`); unused.
    pub reflect_vector: [PlainFloat; 3],
    /// 0x5EA434: no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_56c: PlainDword,
    /// 0x5EA438: `qword_5EA438`, `dword_5EA440`: the normal (`scene::reflect_vector`); now each view's (`scene::view_matrices`); unused.
    pub reflect_normal: [PlainFloat; 3],
    /// 0x5EA444: `flt_5EA444`: the object's turn matrix (`physics::object_turn_matrix`, `physics::object_hits_car`, `race::frame_draw`).
    pub object_turn_matrix: [PlainFloat; 16],
    /// 0x5EA484: `flt_5EA484`: 1092-byte item, only the first 0x40 used as a matrix in place (`race::frame_draw`); now each view's (`scene::view_matrices`); unused.
    pub matrix_scratch: [PlainFloat; 273],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
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
            // an IDA address: wave 3
            object_matrix_ptr: PlainDword::new(0x5EA30C),
            ..ZeroCells::ZERO
        }
    }
}
