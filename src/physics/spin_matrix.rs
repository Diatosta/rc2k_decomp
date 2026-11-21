//! Turns the work car's matrix by its spin over the step. The spin's length
//! (`physics::vector_length` of `spin`) is stored in `spin_length`; when its bits are not 0: the
//! spin as a unit vector into the axis `spin_axis` (`scene::unit_vector`), the angle `spin_angle` =
//! the length times the step `step_time`, the turn's matrix into `turn_matrix`
//! (`physics::axis_turn_matrix`) and squared (`scene::square_matrix`), the crash matrix times it
//! into `turned_matrix` (`scene::multiply_matrices`) and copied back (16 dwords). Then the crash
//! matrix is squared.

use crate::math::matrix3::rounded_value;
use crate::math::x87;
use crate::memory::session::DrawPlain;
use crate::physics::axis_turn_matrix::axis_turn_matrix;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_length::vector_length;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner_course::Course;
use crate::scene::matrix::{matrix_of, product, store_matrix};
use crate::scene::view_matrix::{square_matrix, unit_vector};

pub fn spin_matrix(
    course: &Course,
    matrices: &DrawPlain<SceneMatrices>,
    physics_tables: &PhysicsTables,
    car: &Car,
) {
    car.spin_length.set(rounded_value(vector_length(&car.spin)));
    // Tested as a dword: any bit set (a NaN too) means a spin.
    if car.spin_length.get().to_bits() != 0 {
        let lists = course.lists.get();
        let turn = &lists.turn_matrix;
        unit_vector(course.reads(), &car.spin, &lists.spin_axis);
        lists
            .spin_angle
            .set(x87::mul(&car.spin_length, physics_tables.step_time.get()));
        axis_turn_matrix(course, turn);
        square_matrix(course.reads(), turn);
        let crash = matrix_of(&car.crash_matrix);
        let turn_now = matrix_of(turn);
        let mut kept = matrices.borrow_mut();
        // Only the first three columns are worked out; the fourth stays what the last copy left.
        let crashed = product(&crash, &turn_now, kept.turned_matrix);
        kept.turned_matrix = crashed;
        drop(kept);
        store_matrix(&car.crash_matrix, &crashed);
    }
    square_matrix(course.reads(), &car.crash_matrix);
}
