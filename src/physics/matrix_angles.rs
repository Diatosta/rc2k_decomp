//! `physics::matrix_angles` (0x4212C3): three angles of a turn matrix. The matrix turns (0, 0, 1)
//! (`scene::rotate_point`, stored as floats in `turned_x` ..): `car_heading` is the `fpatan` of
//! the x over the z; the same turn again, and `car_pitch` is minus `physics::clamped_asin` of the
//! y; then (1, 0, 0) is turned, and `car_roll` is `clamped_asin` of the y over the cosine of the
//! pitch (80 bits). A reading: the car's heading, pitch and roll.
//!
//! Takes: the matrix
//!
//! Callers: `physics::car_angles` (`g_crashMatrix`).

use crate::math::cells::Cells;
use crate::math::x87::{neg_rounded, over_cos_80, patan};
use crate::physics::clamped_asin::clamped_asin_at;
use crate::race::owner_course::Course;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::session::{Cell32, Float};

/// `rotate_point` of the unit vector `unit` by `m`, stored as floats in the turned vector.
fn turn_unit(m: &(impl Cells + ?Sized), turned: &[&Float; 3], unit: [u32; 3]) {
    let out = rotate_point(m, unit.map(extended));
    for (cell, e) in turned.iter().zip(out) {
        cell.set_bits(rounded(e));
    }
}

/// On the matrix `m`; the angles in `COURSE_LISTS`.
pub fn matrix_angles(
    course: &Course,
    m: &(impl Cells + ?Sized),
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    const ONE: u32 = 0x3F80_0000;
    let l = &course.lists;
    let turned = [&l.turned_x, &l.turned_y, &l.turned_z];
    turn_unit(m, &turned, [0, 0, ONE]);
    l.car_heading
        .set_bits(patan(l.turned_x.bits(), l.turned_z.bits()).to_bits());
    turn_unit(m, &turned, [0, 0, ONE]);
    let pitch = clamped_asin_at(
        extended(l.turned_y.bits()),
        &drawing.scene.scratch,
        &course.numbers.one,
    );
    l.car_pitch.set_bits(neg_rounded(pitch).to_bits());
    turn_unit(m, &turned, [ONE, 0, 0]);
    let sine = over_cos_80(l.turned_y.bits(), l.car_pitch.bits());
    let roll = clamped_asin_at(sine, &drawing.scene.scratch, &course.numbers.one);
    l.car_roll.set_bits(rounded(roll));
}
