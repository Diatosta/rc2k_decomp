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
use crate::race::owner::Race;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded;
use crate::scene::view_matrix::{square_matrix, unit_vector};

/// The Rust version.
pub fn spin_matrix(race: &Race) {
    CAR.spin_length.set_bits(rounded(vector_length(&CAR.spin)));
    if CAR.spin_length.bits() != 0 {
        let lists = &race.course.lists;
        let turn = &lists.turn_matrix;
        let turned = &race.frame.matrices.turned_matrix;
        unit_vector(&CAR.spin, &lists.spin_axis);
        lists
            .spin_angle
            .set_bits(x87::mul(CAR.spin_length.bits(), PHYSICS.step_time.bits()));
        axis_turn_matrix(&race.course, turn);
        square_matrix(turn);
        multiply_matrices(&CAR.crash_matrix, turn, turned);
        for k in 0..16 {
            CAR.crash_matrix[k].set_bits(race.frame.matrices.turned_matrix[k].bits());
        }
    }
    square_matrix(&CAR.crash_matrix);
}
