//! The steps the physics' long functions share: the x87 compares (the status word's C0 / C3)
//! and the holds and eases whose patterns repeat.

use crate::math::x87;
pub use crate::math::x87::{c0, c3};
use crate::math::x87_float as bh;
use crate::math::x87_float::{compare, ld, stf};
use crate::memory::plain_floats::FloatRow;
use crate::memory::session::{Dword, Float, PlainDword, PlainFloat};

/// The bits of 0.0. The game keeps one such constant in `race.course.numbers.zero`
/// (`numbers.zero`, 0.0); nothing in the program writes that cell, so the
/// physics reads this constant instead of going through the race for it.
pub(super) const ZERO_BITS: u32 = 0;

/// A float cell read and written as the float it holds.
pub trait FloatView {
    fn float(&self) -> f32;
    fn set_float(&self, x: f32);
}

impl FloatView for Float {
    fn float(&self) -> f32 {
        self.get()
    }
    fn set_float(&self, x: f32) {
        self.set(x);
    }
}

impl FloatView for PlainFloat {
    fn float(&self) -> f32 {
        self.get()
    }
    fn set_float(&self, x: f32) {
        self.set(x);
    }
}

impl FloatView for Dword {
    fn float(&self) -> f32 {
        f32::from_bits(self.get())
    }
    fn set_float(&self, x: f32) {
        self.set(x.to_bits());
    }
}

impl FloatView for PlainDword {
    fn float(&self) -> f32 {
        f32::from_bits(self.get())
    }
    fn set_float(&self, x: f32) {
        self.set(x.to_bits());
    }
}

impl<T: FloatView + ?Sized> FloatView for &T {
    fn float(&self) -> f32 {
        (**self).float()
    }
    fn set_float(&self, x: f32) {
        (**self).set_float(x);
    }
}

/// A float place the holds and eases write: a float cell (by reference), or a plain float of a
/// record (by mutable reference).
pub trait FloatPlace {
    fn float(&self) -> f32;
    fn set_float(&mut self, x: f32);
}

impl<T: FloatView + ?Sized> FloatPlace for &T {
    fn float(&self) -> f32 {
        FloatView::float(*self)
    }
    fn set_float(&mut self, x: f32) {
        FloatView::set_float(*self, x);
    }
}

impl FloatPlace for &mut f32 {
    fn float(&self) -> f32 {
        **self
    }
    fn set_float(&mut self, x: f32) {
        **self = x;
    }
}

/// A float read as the value it holds: a cell, or a constant (a limit a value is held to).
pub trait FloatRead {
    fn read_float(&self) -> f32;
}

impl<T: FloatView + ?Sized> FloatRead for T {
    fn read_float(&self) -> f32 {
        self.float()
    }
}

impl FloatRead for f32 {
    fn read_float(&self) -> f32 {
        *self
    }
}

impl FloatRead for &f32 {
    fn read_float(&self) -> f32 {
        **self
    }
}

impl FloatRead for u32 {
    fn read_float(&self) -> f32 {
        f32::from_bits(*self)
    }
}

impl FloatRead for crate::memory::session::ConstFloat {
    fn read_float(&self) -> f32 {
        self.get()
    }
}

/// A word as its bits: a cell, or a plain `u32`, `i32` or `f32` of a record.
pub trait Bits {
    fn word_bits(&self) -> u32;
}

impl<C: crate::memory::session::Cell32 + ?Sized> Bits for C {
    fn word_bits(&self) -> u32 {
        self.bits()
    }
}

impl Bits for u32 {
    fn word_bits(&self) -> u32 {
        *self
    }
}

impl Bits for i32 {
    fn word_bits(&self) -> u32 {
        *self as u32
    }
}

impl Bits for f32 {
    fn word_bits(&self) -> u32 {
        self.to_bits()
    }
}

/// The game's dword in the word `x`.
pub fn g(x: &impl Bits) -> u32 {
    x.word_bits()
}

/// The whole number held in the word `x` (a float variable that holds a count).
pub fn whole(x: &impl Bits) -> i32 {
    x.word_bits() as i32
}

/// The float `a` is below `b`, or one of them is not a number (the x87's C0 after
/// comparing them); `!below_or_unordered(a, b)` is "`a` is at least `b`".
pub fn below_or_unordered(a: impl x87::Flt, b: impl x87::Flt) -> bool {
    c0(a, b)
}

/// The float `a` equals `b`, or one of them is not a number (the x87's C3 after comparing
/// them).
pub fn equal_or_unordered(a: impl x87::Flt, b: impl x87::Flt) -> bool {
    c3(a, b)
}

/// The size of the float `a` is below `b`, or one of them is not a number.
pub fn abs_below_or_unordered(a: impl x87::Flt, b: impl x87::Flt) -> bool {
    crate::math::x87::abs_compare(a, b) & 0x100 != 0
}

/// `v` raised to `lim` when below it (or unordered).
pub fn hold_low(v: impl FloatPlace, lim: &impl FloatRead) {
    hold_low_to(v, lim.read_float());
}

/// `hold_low` against a limit given as a value.
pub(super) fn hold_low_to(mut v: impl FloatPlace, lim: f32) {
    if c0(v.float(), lim) {
        v.set_float(x87::loaded(lim));
    }
}

/// `hold_low` against a limit given as its bits.
pub(super) fn hold_low_bits(v: impl FloatPlace, lim: u32) {
    hold_low_to(v, f32::from_bits(lim));
}

/// `v` lowered to `lim` unless below it (or unordered).
pub fn hold_high(v: impl FloatPlace, lim: &impl FloatRead) {
    hold_high_to(v, lim.read_float());
}

/// `hold_high` against a limit given as a value.
pub(super) fn hold_high_to(mut v: impl FloatPlace, lim: f32) {
    if !c0(v.float(), lim) {
        v.set_float(x87::loaded(lim));
    }
}

/// `hold_low` on a plain float.
pub fn hold_low_plain(v: &mut f32, lim: f32) {
    if c0(*v, lim) {
        *v = x87::loaded(lim);
    }
}

/// `hold_high` on a plain float.
pub fn hold_high_plain(v: &mut f32, lim: f32) {
    if !c0(*v, lim) {
        *v = x87::loaded(lim);
    }
}

/// `v` negated in place.
pub fn negate(mut v: impl FloatPlace) {
    v.set_float(negated(v.float()));
}

/// The car's velocity negated part by part, as the bits of three floats.
pub fn negated_velocity(car: &crate::physics::car_store::Car) -> [u32; 3] {
    std::array::from_fn(|k| negated(car.velocity.at(k)).to_bits())
}

/// `x` negated.
pub fn negated(x: f32) -> f32 {
    crate::math::x87::neg_rounded(crate::math::matrix3::extended(x))
}

/// The value at `v` brought toward 0 by the step at `step`, never past it; 0 (either
/// sign) or unordered left alone.
pub(super) fn ease_zero(mut v: impl FloatPlace, step: f32) {
    let x = v.float();
    if c3(x, 0.0) {
        return;
    }
    if c0(x, 0.0) {
        let y = x87::add(x, step);
        v.set_float(y);
        if !c0(y, 0.0) {
            v.set_float(0.0);
        }
    } else {
        let y = x87::sub(x, step);
        v.set_float(y);
        if c0(y, 0.0) {
            v.set_float(0.0);
        }
    }
}

/// A clamped table lerp (the damage model's): the table at
/// `t` is (x0, y0, x1, y1); the value at `x` below x0 (or unordered) gives y0, below x1
/// gives y0 + (x - x0) / (x1 - x0) x (y1 - y0) (y0 when x1 - x0, unrounded, equals the
/// value at `zero`), else y1. The table is given as the bits of its four figures.
pub(super) fn table_lerp(x: f32, t: [u32; 4], zero: f32) -> f32 {
    if c0(x, t[0]) {
        return x87::loaded(t[1]);
    }
    if !c0(x, t[2]) {
        return x87::loaded(t[3]);
    }
    if x87_1(t, zero) & 0x4000 != 0 {
        return x87::loaded(t[1]);
    }
    x87_2(x, t)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn x87_1(t: [u32; 4], z: f32) -> u16 {
    let t0 = bh::ld(t[2]);
    let t1 = bh::ld(t[0]);
    let t2 = t0 - t1;
    let t3 = ld(z);
    compare(t2, t3)
}

/// One straight x87 run as plain Rust over `X`.
fn x87_2(x: f32, t: [u32; 4]) -> f32 {
    let t0 = ld(x);
    let t1 = bh::ld(t[0]);
    let t2 = t0 - t1;
    let t3 = bh::ld(t[2]);
    let t4 = bh::ld(t[0]);
    let t5 = t3 - t4;
    let t6 = t2 / t5;
    let t7 = bh::ld(t[3]);
    let t8 = bh::ld(t[1]);
    let t9 = t7 - t8;
    let t10 = t6 * t9;
    let t11 = bh::ld(t[1]);
    let t12 = t10 + t11;
    stf(t12)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_mem::{Base, view};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "flow line 127",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F), (8, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    a[2].set_word(
                        0,
                        x87_1(
                            view::<[PlainFloat; 4]>(a[0], 0)
                                .each_ref()
                                .map(PlainFloat::bits),
                            view::<Float>(a[1], 0).get(),
                        ),
                    );
                },
            },
            Block {
                name: "flow line 138",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F), (4, In::F), (8, In::F), (0xC, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    view::<Float>(a[2], 0).set(x87_2(
                        view::<Float>(a[0], 0).get(),
                        view::<[PlainFloat; 4]>(a[1], 0)
                            .each_ref()
                            .map(PlainFloat::bits),
                    ));
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.flow", &blocks(), 1200, 100);
    }
}
