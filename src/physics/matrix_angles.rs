//! Three angles of a turn matrix. The matrix turns (0, 0, 1) (`scene::rotate_point`, stored as
//! floats in `turned_x` ..): `car_heading` is the arctangent of the x over the z; the same turn
//! again, and `car_pitch` is minus `physics::clamped_asin` of the y; then (1, 0, 0) is turned, and
//! `car_roll` is `clamped_asin` of the y over the cosine of the pitch (80 bits). A reading: the
//! car's heading, pitch and roll.
//!
//! Caller: `physics::car_angles`, with the crash matrix.

use crate::math::cells::Cells;
use crate::math::x87::{neg_rounded, over_cos_80, patan};
use crate::memory::session::PlainFloat;
use crate::physics::clamped_asin::clamped_asin_at;
use crate::race::owner_course::Course;
use crate::scene::matrix3::{extended, rounded_value};
use crate::scene::rotate_point::rotate_point;

/// `rotate_point` of the unit vector `unit` by `m`, stored as floats in the turned vector.
fn turn_unit(m: &(impl Cells + ?Sized), turned: &[&PlainFloat; 3], unit: [f32; 3]) {
    let out = rotate_point(m, unit.map(extended));
    for (cell, e) in turned.iter().zip(out) {
        cell.set(rounded_value(e));
    }
}

/// On the matrix `m`; the angles go into the course lists (`car_heading`, `car_pitch`, `car_roll`).
pub fn matrix_angles(
    course: &Course,
    m: &(impl Cells + ?Sized),
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    let l = course.lists.get();
    let turned = [&l.turned_x, &l.turned_y, &l.turned_z];
    turn_unit(m, &turned, [0.0, 0.0, 1.0]);
    l.car_heading.set(patan(&l.turned_x, &l.turned_z));
    turn_unit(m, &turned, [0.0, 0.0, 1.0]);
    let pitch = clamped_asin_at(
        extended(l.turned_y.get()),
        &drawing.scene.get().scratch,
        &course.numbers.get().one,
    );
    l.car_pitch.set(neg_rounded(pitch));
    turn_unit(m, &turned, [1.0, 0.0, 0.0]);
    let sine = over_cos_80(&l.turned_y, &l.car_pitch);
    let roll = clamped_asin_at(
        sine,
        &drawing.scene.get().scratch,
        &course.numbers.get().one,
    );
    l.car_roll.set(rounded_value(roll));
}
