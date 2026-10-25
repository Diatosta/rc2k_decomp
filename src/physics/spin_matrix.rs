//! `physics::spin_matrix` (0x43DD54): the work car's matrix turned by its spin over the
//! step. The spin's length (`physics::vector_length` of `dword_71A15C` .., `g_car`
//! +0x184) is stored in `flt_71A16C`; when its bits are not 0 (`cmp` as a dword): the
//! spin as a unit vector into the axis `flt_5C78FC` (`scene::unit_vector`), the angle
//! `flt_5C78F8` = the length times the step `flt_61388C`, the turn's matrix into
//! `flt_5C7010` (`physics::axis_turn_matrix`) and squared (`scene::square_matrix`),
//! `g_crashMatrix` times it into `g_turnedMatrix` (`scene::multiply_matrices`) and copied
//! back (16 dwords, `rep movsd` in `pusha` / `popa`). Then `g_crashMatrix` squared.
//!
//! In:  nothing
//! Out: esi `g_crashMatrix`; when the spin is not 0, also ecx and edx 0 (`multiply_
//!      matrices`), edi `flt_5C7010`, ebp `g_turnedMatrix`; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B82C, 0x43B8A5), every physics step.

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded;
use crate::scene::view_matrix::{square_matrix, unit_vector};

fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const AXIS: usize = 0x5C78FC; // flt_5C78FC
const ANGLE: usize = 0x5C78F8; // flt_5C78F8
const TURN: usize = 0x5C7010; // flt_5C7010
const TURNED: usize = 0x5EA10C; // g_turnedMatrix

/// The Rust version.
pub unsafe fn spin_matrix() {
    unsafe {
        CAR.spin_length
            .set_bits(rounded(vector_length(CAR.spin[0].addr() as u32)));
        if CAR.spin_length.bits() != 0 {
            unit_vector((CAR.spin[0].addr() as u32) as usize, a(AXIS) as usize);
            p(
                ANGLE,
                x87::mul(CAR.spin_length.bits(), PHYSICS.step_time.bits()),
            );
            axis_turn_matrix(a(TURN));
            square_matrix(a(TURN) as usize);
            multiply_matrices(
                (CAR.crash_matrix[0].addr() as u32) as usize,
                a(TURN) as usize,
                a(TURNED) as usize,
            );
            for k in 0..16 {
                CAR.crash_matrix[k].set_bits(g(TURNED + 4 * k));
            }
        }
        square_matrix((CAR.crash_matrix[0].addr() as u32) as usize);
    }
}
