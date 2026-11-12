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
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_length::vector_length;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner_course::Course;
use crate::scene::matrix::multiply_matrices;
use crate::scene::matrix3::rounded_value;
use crate::scene::view_matrix::{square_matrix, unit_vector};

pub fn spin_matrix(
    course: &Course,
    matrices: &SceneMatrices,
    physics_tables: &PhysicsTables,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) {
    car.spin_length.set(rounded_value(vector_length(&car.spin)));
    // Tested as a dword: any bit set (a NaN too) means a spin.
    if car.spin_length.get().to_bits() != 0 {
        let lists = course.lists.get();
        let turn = &lists.turn_matrix;
        let turned = &matrices.turned_matrix;
        unit_vector(course.reads(), &car.spin, &lists.spin_axis);
        lists
            .spin_angle
            .set(x87::mul(&car.spin_length, &physics_tables.step_time));
        axis_turn_matrix(course, turn);
        square_matrix(course.reads(), turn);
        multiply_matrices(&car.crash_matrix, turn, turned);
        for k in 0..16 {
            car.crash_matrix[k].set(matrices.turned_matrix[k].get());
        }
    }
    // `scene::square_matrix` left its last length in `g_scratch` (the ones before are
    // written over by it).
    let length = square_matrix(course.reads(), &car.crash_matrix);
    scene.drawing.scene.get().scratch.set(length);
}
