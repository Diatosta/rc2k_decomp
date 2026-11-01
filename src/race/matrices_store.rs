//! The scene's matrices in the race block, 0x5E9EC8 .. 0x5EA8C8: the identity, the three turn matrices, the placed / turned / lit matrices, the view matrices, the object matrices and a scratch area.
//! Most are 4x4 float matrices (64 bytes) that Rust reaches in place by address (`ga(..)`) and passes to matrix functions; the generated struct overlays the same bytes.
//! 0x5EA484 is IDA's one 1092-byte item, taken whole; only its first 64 bytes are used.
//! Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SceneMatrices {
    /// +0x0 (0x5E9EC8): `dword_5E9EC8`: the model last set up for drawing (`scene::load_object`, `race::car_draw`).
    pub known_object: Dword,
    /// +0x4 (0x5E9ECC): `flt_5E9ECC`: two matrices of the frame draw, used in place as scratch (`race::frame_draw`).
    pub view_work: [Float; 32],
    /// +0x84 (0x5E9F4C): `g_identityMatrix`: the 4x4 identity (`scene::view_matrix`, `race::frame_draw`).
    pub identity_matrix: [Float; 16],
    /// +0xc4 (0x5E9F8C): `g_turnXMatrix`: rotation about x (`scene::matrix`, `physics::object_turn_matrix`).
    pub turn_x_matrix: [Float; 16],
    /// +0x104 (0x5E9FCC): `g_turnYMatrix`: rotation about y (`scene::matrix`, `race::car_parts`).
    pub turn_y_matrix: [Float; 16],
    /// +0x144 (0x5EA00C): `g_turnZMatrix`: rotation about z (`scene::view_matrix`, `race::car_draw`).
    pub turn_z_matrix: [Float; 16],
    /// +0x184 (0x5EA04C): `g_placedMatrix`: a matrix with its place (`scene::matrix`, `physics::car_collisions`).
    pub placed_matrix: [Float; 16],
    /// +0x1c4 (0x5EA08C): `flt_5EA08C`: the car body's turn matrix copied from the car (`race::car_parts`, `race::car_draw`).
    pub body_turn_matrix: [Float; 16],
    /// +0x204 (0x5EA0CC): unnamed buffer used in place (`drive::set_listener`, `effects::draw_shadow`) (a reading).
    pub listener_matrix: [Float; 16],
    /// +0x244 (0x5EA10C): `g_turnedMatrix`: the turned matrix (`race::frame_draw`, `physics::move_slot`, `scene::matrix`).
    pub turned_matrix: [Float; 16],
    /// +0x284 (0x5EA14C): `g_litMatrix`: the lit matrix (`race::frame_draw`, `race::car_parts`).
    pub lit_matrix: [Float; 16],
    /// +0x2c4 (0x5EA18C): `flt_5EA18C`: the inverted view turn (`race::frame_draw`).
    pub view_turn_copy: [Float; 16],
    /// +0x304 (0x5EA1CC): unnamed buffer used in place (`race::frame_draw`, `scene::view_matrix`) (a reading).
    pub view_place_matrix: [Float; 16],
    /// +0x344 (0x5EA20C): `g_viewBackMatrix`: the view turned back (`scene::view_matrix`, `scene::reflect_vector`).
    pub view_back_matrix: [Float; 16],
    /// +0x384 (0x5EA24C): `g_viewTurnMatrix`: the view's turn (`drive::set_listener`, `physics::car_sounds`, `race::frame_draw`).
    pub view_turn_matrix: [Float; 16],
    /// +0x3c4 (0x5EA28C): not named.
    pub unknown_3c4: [Dword; 16],
    /// +0x404 (0x5EA2CC): `g_viewMatrix` and its elements `flt_5EA2D0`..: the view matrix, used in place (`scene::project_point`, `effects::draw_particle`, `race::skidmarks`).
    pub view_matrix: [Float; 16],
    /// +0x444 (0x5EA30C): `g_madeObjectMatrix`: the matrix made for an object (`scene::matrix`, `race::frame_draw`).
    pub made_object_matrix: [Float; 16],
    /// +0x484 (0x5EA34C): `dword_5EA34C`: a block whose first 0x40 bytes receive a copy of the made object matrix (`race::frame_draw`).
    pub object_matrix_slots: [Dword; 32],
    /// +0x504 (0x5EA3CC): `flt_5EA3CC`: the view's matrix, place at +0x30 (`effects::view_motion`, `race::frame_views`).
    pub view_place: [Float; 16],
    /// +0x544 (0x5EA40C): `g_objectMatrix`: pointer to the object matrix in use (`scene::place_object_points`).
    pub object_matrix_ptr: Dword,
    /// +0x548 (0x5EA410): not named.
    pub unknown_548: [Dword; 6],
    /// +0x560 (0x5EA428): `dword_5EA428`..: the vector reflected (`scene::reflect_vector`).
    pub reflect_vector: [Float; 3],
    /// +0x56c (0x5EA434): not named.
    pub unknown_56c: Dword,
    /// +0x570 (0x5EA438): `qword_5EA438`, `dword_5EA440`: the normal (`scene::reflect_vector`).
    pub reflect_normal: [Float; 3],
    /// +0x57c (0x5EA444): `flt_5EA444`: the object's turn matrix (`physics::object_turn_matrix`, `physics::object_hits_car`, `race::frame_draw`).
    pub object_turn_matrix: [Float; 16],
    /// +0x5bc (0x5EA484): `flt_5EA484`: 1092-byte item, only the first 0x40 used as a matrix in place (`race::frame_draw`).
    pub matrix_scratch: [Float; 273],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static MATRICES: SceneMatrices =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x5E9EC8, the first byte of the piece.
pub const START: usize = 0x5E9EC8;
/// IDA 0x5EA8C8, the end of the piece.
pub const END: usize = 0x5EA8C8;

const _: () = assert!(END - START == core::mem::size_of::<SceneMatrices>());

/// Where IDA 0x5E9EC8 is now (its own static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&MATRICES))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&MATRICES, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(SceneMatrices, known_object), 0x5E9EC8 - START);
        assert_eq!(offset_of!(SceneMatrices, placed_matrix), 0x5EA04C - START);
        assert_eq!(
            offset_of!(SceneMatrices, view_place_matrix),
            0x5EA1CC - START
        );
        assert_eq!(offset_of!(SceneMatrices, view_place), 0x5EA3CC - START);
    }
}
