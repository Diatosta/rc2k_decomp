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
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): spins by lot and 0, turn matrices, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EBP, EDI, ESI, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded;
use crate::scene::view_matrix::{square_matrix, unit_vector};
use crate::screens::harness::Left;

fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const SPIN: usize = 0x71A15C; // g_car + 0x184
const SPIN_LENGTH: usize = 0x71A16C; // flt_71A16C
const AXIS: usize = 0x5C78FC; // flt_5C78FC
const ANGLE: usize = 0x5C78F8; // flt_5C78F8
const TURN: usize = 0x5C7010; // flt_5C7010
const CRASH: usize = 0x71A08C; // g_crashMatrix
const TURNED: usize = 0x5EA10C; // g_turnedMatrix

/// The Rust version.
pub unsafe fn spin_matrix() -> Left {
    unsafe {
        let mut left = Left::nothing();
        p(SPIN_LENGTH, rounded(vector_length(a(SPIN))));
        if g(SPIN_LENGTH) != 0 {
            unit_vector(a(SPIN) as usize, a(AXIS) as usize);
            p(ANGLE, x87::mul(g(SPIN_LENGTH), g(0x61388C)));
            axis_turn_matrix(a(TURN));
            square_matrix(a(TURN) as usize);
            left = multiply_matrices(a(CRASH) as usize, a(TURN) as usize, a(TURNED) as usize)
                .with(EDI, a(TURN))
                .with(EBP, a(TURNED));
            for k in 0..16 {
                p(CRASH + 4 * k, g(TURNED + 4 * k));
            }
        }
        square_matrix(a(CRASH) as usize);
        left.with(ESI, a(CRASH))
    }
}
