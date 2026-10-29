//! `physics::axis_turn_matrix` (0x421019): the matrix at edi of a turn by `flt_5C78F8`
//! about the axis `flt_5C78FC` .. `flt_5C7904` (Rodrigues' form, with |x| square-rooted
//! for the diagonal's x x, as the original has it), its work values left in
//! `flt_5C7098` .. `flt_5C70B8` (sine, cosine, 1 - cosine, three products of two axis
//! parts and three of one with the sine); the fourth row 0, 0, 0, 1; the fourth column's
//! three left as they were.
//!
//! In:  edi the matrix (0x40 bytes)
//! Out: every register kept; no flags read (callers go on with `push`, `lea`)
//!
//! Callers: `physics::move_slot` (0x4624D6) and `sub_43DD54` (0x43DD97).

use crate::math::cells::Cells;
use crate::math::x87;
use crate::math::x87_run::{abs_sqrt_times_plus, one_less, sine_cosine_of};
use crate::race::course_lists_store::COURSE_LISTS;

/// The Rust version: the matrix into `out` (16 cells).
pub fn axis_turn_matrix(out: &(impl Cells + ?Sized)) {
    {
        let (s, c) = sine_cosine_of(COURSE_LISTS.spin_angle.bits());
        COURSE_LISTS.turn_cosine.set_bits(c);
        COURSE_LISTS.turn_sine.set_bits(s);
        let t = one_less(COURSE_LISTS.turn_cosine.bits());
        COURSE_LISTS.turn_one_less_cosine.set_bits(t);
        let (x, y, z) = (
            COURSE_LISTS.spin_axis[0].bits(),
            COURSE_LISTS.spin_axis[1].bits(),
            COURSE_LISTS.spin_axis[2].bits(),
        );
        COURSE_LISTS.turn_product_xz.set_bits(x87::mul2(t, x, z));
        COURSE_LISTS.turn_product_xy.set_bits(x87::mul2(t, x, y));
        COURSE_LISTS.turn_product_yz.set_bits(x87::mul2(t, y, z));
        COURSE_LISTS.turn_sine_x.set_bits(x87::mul(x, s));
        COURSE_LISTS.turn_sine_y.set_bits(x87::mul(y, s));
        COURSE_LISTS.turn_sine_z.set_bits(x87::mul(z, s));
        let (p1, p2, p3) = (
            COURSE_LISTS.turn_product_xz.bits(),
            COURSE_LISTS.turn_product_xy.bits(),
            COURSE_LISTS.turn_product_yz.bits(),
        );
        let (q1, q2, q3) = (
            COURSE_LISTS.turn_sine_x.bits(),
            COURSE_LISTS.turn_sine_y.bits(),
            COURSE_LISTS.turn_sine_z.bits(),
        );
        let put = |o: usize, v: u32| out.set_bits(o / 4, v);
        put(0x00, abs_sqrt_times_plus(x, t, c));
        put(0x04, x87::add(p2, q3));
        put(0x08, x87::sub(p1, q2));
        put(0x10, x87::sub(p2, q3));
        put(0x14, abs_sqrt_times_plus(y, t, c));
        put(0x18, x87::add(p3, q1));
        put(0x20, x87::add(p1, q2));
        put(0x24, x87::sub(p3, q1));
        put(0x28, abs_sqrt_times_plus(z, t, c));
        for o in [0x30, 0x34, 0x38] {
            put(o, 0);
        }
        put(0x3C, x87::loaded(1.0f32.to_bits()));
    }
}
