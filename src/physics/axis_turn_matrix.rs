//! Fills a matrix with a turn by `spin_angle` about the axis `spin_axis` (Rodrigues' form, with
//! the diagonal's x x square-rooted from its absolute value). The work values (sine, cosine,
//! 1 - cosine, three products of two axis parts and three of one with the sine) are left in
//! `turn_sine` .. `turn_sine_z`; the fourth row is 0, 0, 0, 1 and the fourth column is left as
//! it was.

use crate::math::cells::Cells;
use crate::math::x87;
use crate::math::x87::{abs_sqrt_times_plus, one_less, sine_cosine_of};
use crate::race::owner_course::Course;

/// The matrix into `out` (16 cells).
pub fn axis_turn_matrix(course: &Course, out: &(impl Cells + ?Sized)) {
    {
        let (s, c) = sine_cosine_of(&course.turn_work().spin_angle);
        course.turn_work().turn_cosine.set_bits(c);
        course.turn_work().turn_sine.set_bits(s);
        let t = one_less(&course.turn_work().turn_cosine);
        course.turn_work().turn_one_less_cosine.set(t);
        let (x, y, z) = (
            &course.turn_work().spin_axis[0],
            &course.turn_work().spin_axis[1],
            &course.turn_work().spin_axis[2],
        );
        course.turn_work().turn_product_xz.set(x87::mul2(t, x, z));
        course.turn_work().turn_product_xy.set(x87::mul2(t, x, y));
        course.turn_work().turn_product_yz.set(x87::mul2(t, y, z));
        course.turn_work().turn_sine_x.set(x87::mul(x, s));
        course.turn_work().turn_sine_y.set(x87::mul(y, s));
        course.turn_work().turn_sine_z.set(x87::mul(z, s));
        let (p1, p2, p3) = (
            &course.turn_work().turn_product_xz,
            &course.turn_work().turn_product_xy,
            &course.turn_work().turn_product_yz,
        );
        let (q1, q2, q3) = (
            &course.turn_work().turn_sine_x,
            &course.turn_work().turn_sine_y,
            &course.turn_work().turn_sine_z,
        );
        let put = |o: usize, v: u32| out.set_bits(o / 4, v);
        put(0x00, abs_sqrt_times_plus(x, t, c).to_bits());
        put(0x04, x87::add(p2, q3).to_bits());
        put(0x08, x87::sub(p1, q2).to_bits());
        put(0x10, x87::sub(p2, q3).to_bits());
        put(0x14, abs_sqrt_times_plus(y, t, c).to_bits());
        put(0x18, x87::add(p3, q1).to_bits());
        put(0x20, x87::add(p1, q2).to_bits());
        put(0x24, x87::sub(p3, q1).to_bits());
        put(0x28, abs_sqrt_times_plus(z, t, c).to_bits());
        for o in [0x30, 0x34, 0x38] {
            put(o, 0);
        }
        put(0x3C, x87::loaded(1.0f32.to_bits()).to_bits());
    }
}
