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

use crate::math::x87;
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::physics::vector_length::vector_length;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::matrices_store::MATRICES;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded;
use crate::scene::view_matrix::{square_matrix, unit_vector};

/// The Rust version.
pub unsafe fn spin_matrix() {
    unsafe {
        CAR.spin_length
            .set_bits(rounded(vector_length(CAR.spin[0].addr() as u32)));
        if CAR.spin_length.bits() != 0 {
            let lists = &*COURSE_LISTS;
            let turn = lists.turn_matrix[0].addr();
            let turned = MATRICES.turned_matrix[0].addr();
            unit_vector(
                (CAR.spin[0].addr() as u32) as usize,
                lists.spin_axis_x.addr(),
            );
            lists
                .spin_angle
                .set_bits(x87::mul(CAR.spin_length.bits(), PHYSICS.step_time.bits()));
            axis_turn_matrix(turn as u32);
            square_matrix(turn);
            multiply_matrices((CAR.crash_matrix[0].addr() as u32) as usize, turn, turned);
            for k in 0..16 {
                CAR.crash_matrix[k].set_bits(MATRICES.turned_matrix[k].bits());
            }
        }
        square_matrix((CAR.crash_matrix[0].addr() as u32) as usize);
    }
}
