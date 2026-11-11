//! `physics::spin_matrix` (0x43DD54): the work car's matrix turned by its spin over the
//! step. The spin's length (`physics::vector_length` of `spin` .., `g_car`
//! +0x184) is stored in `spin_length`; when its bits are not 0 (`cmp` as a dword): the
//! spin as a unit vector into the axis `spin_axis` (`scene::unit_vector`), the angle
//! `spin_angle` = the length times the step `step_time`, the turn's matrix into
//! `turn_matrix` (`physics::axis_turn_matrix`) and squared (`scene::square_matrix`),
//! `g_crashMatrix` times it into `g_turnedMatrix` (`scene::multiply_matrices`) and copied
//! back (16 dwords, `rep movsd`). Then `g_crashMatrix` squared.
//!
//! Callers: `physics::car_physics_step` (0x43B82C, 0x43B8A5), every physics step.

use crate::math::x87;
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::vector_length::vector_length;
use crate::race::owner::Race;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded;
use crate::scene::view_matrix::{square_matrix, unit_vector};

pub fn spin_matrix(race: &Race) {
    race.shared
        .car
        .spin_length
        .set_bits(rounded(vector_length(&race.shared.car.spin)));
    if race.shared.car.spin_length.bits() != 0 {
        let lists = &race.course.lists;
        let turn = &lists.turn_matrix;
        let turned = &race.frame.matrices.turned_matrix;
        unit_vector(&race.course, &race.shared.car.spin, &lists.spin_axis);
        lists.spin_angle.set(x87::mul(
            &race.shared.car.spin_length,
            &race.shared.physics_tables.step_time,
        ));
        axis_turn_matrix(&race.course, turn);
        square_matrix(&race.course, turn);
        multiply_matrices(&race.shared.car.crash_matrix, turn, turned);
        for k in 0..16 {
            race.shared.car.crash_matrix[k].set_bits(race.frame.matrices.turned_matrix[k].bits());
        }
    }
    // `scene::square_matrix` left its last length in `g_scratch` (the ones before are
    // written over by it).
    let length = square_matrix(&race.course, &race.shared.car.crash_matrix);
    crate::scene::owner::scene()
        .drawing
        .scene
        .scratch
        .set_bits(length);
}
