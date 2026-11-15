//! The steps the physics' long functions share: the x87 compares (the status word's C0 / C3)
//! and the holds and eases whose patterns repeat.

use crate::effects::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87::compare_full;
use crate::math::x87_float::{compare, ld, stf};
use crate::memory::session::{Dword, Float, PlainDword, PlainFloat};

/// The bits of 0.0. The game keeps one such constant in `race.course.numbers.zero`
/// (`numbers.zero`, 0.0); nothing in the program writes that cell, so the
/// physics reads this constant instead of going through the race for it.
pub const ZERO_BITS: u32 = 0;

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

/// A dword cell used for a float as well as for whole numbers (the scratch
/// cells, the work vectors): its bits read and written as the float.
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

/// The game's dword in the cell `x`.
pub fn g(x: &impl crate::memory::session::Cell32) -> u32 {
    x.bits()
}

/// Writes the dword `v` into the cell `x`.
pub fn p(x: &impl crate::memory::session::Cell32, v: u32) {
    x.set_bits(v);
}

/// The whole number held in the cell `x` (a float variable that holds a count).
pub fn whole(x: &impl crate::memory::session::Cell32) -> i32 {
    x.bits() as i32
}

/// Stores the whole number `v` into the cell `x`.
pub fn set_whole(x: &impl crate::memory::session::Cell32, v: i32) {
    x.set_bits(v as u32);
}

/// The x87's C0 after comparing `a` with `b`: `a` below `b` or unordered.
pub fn c0(a: impl x87::Flt, b: impl x87::Flt) -> bool {
    compare_full(a.bits(), b.bits()) & 0x100 != 0
}

/// The x87's C3 after comparing `a` with `b`: `a` equal to `b` or unordered.
pub fn c3(a: impl x87::Flt, b: impl x87::Flt) -> bool {
    compare_full(a.bits(), b.bits()) & 0x4000 != 0
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
pub fn hold_low(v: &impl FloatView, lim: &impl FloatView) {
    hold_low_to(v, lim.float());
}

/// `hold_low` against a limit given as a value.
pub fn hold_low_to(v: &impl FloatView, lim: f32) {
    if c0(v.float(), lim) {
        v.set_float(x87::loaded(lim));
    }
}

/// `hold_low` against a limit given as its bits.
pub fn hold_low_bits(v: &impl FloatView, lim: u32) {
    hold_low_to(v, f32::from_bits(lim));
}

/// `v` lowered to `lim` unless below it (or unordered).
pub fn hold_high(v: &impl FloatView, lim: &impl FloatView) {
    if !c0(v.float(), lim.float()) {
        v.set_float(x87::loaded(lim.float()));
    }
}

/// `v` negated in place.
pub fn negate(v: &impl FloatView) {
    v.set_float(crate::math::x87::neg_rounded(
        crate::scene::matrix3::extended(v.float()),
    ));
}

/// The value at `v` brought toward 0 by the step at `step`, never past it; 0 (either
/// sign) or unordered left alone.
pub fn ease_zero(v: &impl FloatView, step: &impl FloatView) {
    let x = v.float();
    if c3(x, 0.0) {
        return;
    }
    if c0(x, 0.0) {
        let y = x87::add(x, step.float());
        v.set_float(y);
        if !c0(y, 0.0) {
            v.set_float(0.0);
        }
    } else {
        let y = x87::sub(x, step.float());
        v.set_float(y);
        if c0(y, 0.0) {
            v.set_float(0.0);
        }
    }
}

/// A clamped table lerp (the damage model's): the table at
/// `t` is (x0, y0, x1, y1); the value at `x` below x0 (or unordered) gives y0, below x1
/// gives y0 + (x - x0) / (x1 - x0) x (y1 - y0) (y0 when x1 - x0, unrounded, equals the
/// value at `zero`), else y1; into `out`.
pub fn table_lerp(
    x: &impl FloatView,
    t: &[PlainFloat; 4],
    zero: &impl FloatView,
    out: &impl FloatView,
) {
    if c0(x.float(), &t[0]) {
        out.set_float(x87::loaded(&t[1]));
        return;
    }
    if !c0(x.float(), &t[2]) {
        out.set_float(x87::loaded(&t[3]));
        return;
    }
    if x87_1(t, zero.float()) & 0x4000 != 0 {
        out.set_float(x87::loaded(&t[1]));
        return;
    }
    out.set_float(x87_2(x.float(), t));
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn x87_1(t: &[PlainFloat; 4], z: f32) -> u16 {
    let t0 = t[2].ld();
    let t1 = t[0].ld();
    let t2 = t0 - t1;
    let t3 = ld(z);
    compare(t2, t3)
}

/// One straight x87 run as plain Rust over `X`.
fn x87_2(x: f32, t: &[PlainFloat; 4]) -> f32 {
    let t0 = ld(x);
    let t1 = t[0].ld();
    let t2 = t0 - t1;
    let t3 = t[2].ld();
    let t4 = t[0].ld();
    let t5 = t3 - t4;
    let t6 = t2 / t5;
    let t7 = t[3].ld();
    let t8 = t[1].ld();
    let t9 = t7 - t8;
    let t10 = t6 * t9;
    let t11 = t[1].ld();
    let t12 = t10 + t11;
    stf(t12)
}

/// The first three cells of `from` copied into the first three of `to` (three dword moves).
pub fn copy_cells(to: &[impl FloatView], from: &[impl FloatView]) {
    for k in 0..3 {
        to[k].set_float(from[k].float());
    }
}

/// The first three cells of `to` set to `v` (three dword moves).
pub fn fill_cells(to: &[impl FloatView], v: f32) {
    for cell in &to[..3] {
        cell.set_float(v);
    }
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::scene::x87_mem::{Base, view};
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
                plain: |a| a[2].set_word(0, x87_1(view(a[0], 0), view::<Float>(a[1], 0).get())),
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
                    view::<Float>(a[2], 0).set(x87_2(view::<Float>(a[0], 0).get(), view(a[1], 0)));
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.flow", &blocks(), 1200, 100);
    }
}
