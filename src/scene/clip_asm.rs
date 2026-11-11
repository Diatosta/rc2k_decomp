//! The x87 arithmetic of the clippers, in plain Rust over `X` (`math/x87_float.rs`), converted
//! by hand from the original's listing. Every operation is the
//! original's, in its order, with its operands in a `Work` block instead of the game's
//! entries, records and work variables (a store that a later one overwrites unread is
//! left out). A byte the original loads through `SceneDraw.scratch` is in the block as a
//! whole number already. The original's instructions are `asm_reference` in the tests.

use crate::math::x87_float::X;
use crate::scene::x87_mem::{Base, cells, fild, fistp, fld, fstp};

/// The block the arithmetic works in, in dwords. Offsets: 0 the first point's record,
/// 0x20 the second's, 0x40 and 0x88 their entries, 0xD0 and 0xE4 the entries' five
/// bytes (+0x18, +0x19, +0x1A, +0x20, +0x21) as dwords, 0xF8 the work variables
/// `SceneDraw.clip_work[0]` .. `SceneDraw.clip_work[11]`, 0x128 `SceneDraw.clip_out_words[0]` .. `SceneDraw.clip_out_words[4]`, 0x13C
/// `SceneDraw.scratch`, 0x140 `SceneDraw.clip_plane`.
pub type Work = [u32; 0x51];
pub const RECORD_A: usize = 0x00;
pub const RECORD_B: usize = 0x20;
pub const ENTRY_A: usize = 0x40;
pub const ENTRY_B: usize = 0x88;
pub const BYTES_A: usize = 0xD0;
pub const BYTES_B: usize = 0xE4;
pub const OUT_1: usize = 0xF8;
pub const OUT_2: usize = 0x128;
pub const CHECKED: usize = 0x13C;
pub const PLANE: usize = 0x140;

/// 0x44D247 from after its `fstp flt_623F80` (the edge is in the block already) to its `call`.
pub fn cut_at_x(w: &mut Work) {
    cut_axis(cells(w), X_AXIS);
}

/// 0x44D532, likewise with `flt_623F84`.
pub fn cut_at_y(w: &mut Work) {
    cut_axis(cells(w), Y_AXIS);
}

/// Where a clipper finds the axis it cuts on, as byte offsets into the block: the edge value
/// (and the slot where the other axis' edge value is stored), the cut axis' float in each
/// point's record, and the other axis' float in each record.
#[derive(Clone, Copy)]
struct Axis {
    edge: usize,
    other_edge: usize,
    a: usize,
    b: usize,
    other_a: usize,
    other_b: usize,
}

const X_AXIS: Axis = Axis {
    edge: 0xF8,
    other_edge: 0xFC,
    a: 0x0C,
    b: 0x2C,
    other_a: 0x1C,
    other_b: 0x3C,
};
const Y_AXIS: Axis = Axis {
    edge: 0xFC,
    other_edge: 0xF8,
    a: 0x1C,
    b: 0x3C,
    other_a: 0x0C,
    other_b: 0x2C,
};

/// Cuts the segment between the two points at the edge of one axis: the fraction along it
/// (`SceneDraw.scratch` takes it rounded), every interpolated record field and entry byte, and
/// the other axis' edge value.
fn cut_axis(w: impl Base, axis: Axis) {
    let depth_a = fld(w, 0x18);
    let depth_b = fld(w, 0x38);
    let edge = fld(w, axis.edge);
    let along_a = fld(w, axis.a);
    let along_b = fld(w, axis.b);
    let t4 = edge * (depth_a - depth_b);
    let t8 = -(depth_a * along_a);
    let t9 = t8 + t4;
    let t12 = depth_b * along_b;
    let t13 = t12 + t9;
    let t14 = fld(w, axis.edge);
    let t15 = fld(w, axis.b);
    let t18 = fld(w, 0x38) * (t15 - t14);
    let fraction = t18 / t13;
    interpolate_cut(w, fraction);
    let span = fld(w, axis.b) - fld(w, axis.a);
    let from_edge = fld(w, axis.a) - fld(w, axis.edge);
    let other_a = fld(w, axis.other_a);
    let other_span = fld(w, axis.other_b) - other_a;
    let t104 = other_span * -from_edge;
    let t105 = t104 / span;
    fstp(w, axis.other_edge, t105 + other_a);
}

/// The part every clipper shares: the cut point's record fields (floats and integers), the
/// entry's floats and the five entry bytes, each `a + (b - a) * fraction`, then the fraction
/// itself as an integer in `SceneDraw.scratch`.
fn interpolate_cut(w: impl Base, fraction: X) {
    for (from, to, store) in [
        (0x30, 0x10, 0x100),
        (0x34, 0x14, 0x104),
        (0x38, 0x18, 0x108),
    ] {
        let a = fld(w, from);
        let b = fld(w, to);
        fstp(w, store, a + (b - a) * fraction);
    }
    for (from, to, store) in [
        (0x20, 0x00, 0x10C),
        (0x24, 0x04, 0x110),
        (0x28, 0x08, 0x114),
    ] {
        let a = fild(w, from);
        let b = fild(w, to);
        fistp(w, store, a + (b - a) * fraction);
    }
    for (from, to, store) in [
        (0x90, 0x48, 0x118),
        (0x94, 0x4C, 0x11C),
        (0x98, 0x50, 0x120),
        (0x9C, 0x54, 0x124),
    ] {
        let a = fld(w, from);
        let b = fld(w, to);
        fstp(w, store, a + (b - a) * fraction);
    }
    for (from, to, store) in [
        (0xE4, 0xD0, 0x130),
        (0xE8, 0xD4, 0x134),
        (0xEC, 0xD8, 0x138),
        (0xF0, 0xDC, 0x128),
        (0xF4, 0xE0, 0x12C),
    ] {
        let a = fild(w, from);
        let b = fild(w, to);
        fistp(w, store, a + (b - a) * fraction);
    }
    fistp(w, 0x13C, fraction);
}

/// 0x44DB93 from 0x44DC37 to 0x44DD9A (0x44DEAA has the same there).
pub fn cut_at_depth(w: &mut Work) {
    cut_depth(cells(w));
}

fn cut_depth(w: impl Base) {
    let t0 = fild(w, 0x140);
    let t1 = fld(w, 0x18);
    let t2 = t0 - t1;
    let t3 = fld(w, 0x38);
    let t4 = fld(w, 0x18);
    let t5 = t3 - t4;
    let t6 = t2 / t5;
    let t7 = fld(w, 0x10);
    let t8 = fld(w, 0x30);
    let t9 = t8 - t7;
    let t10 = t9 * t6;
    let t11 = t7 + t10;
    fstp(w, 0x100, t11);
    let t12 = fld(w, 0x14);
    let t13 = fld(w, 0x34);
    let t14 = t13 - t12;
    let t15 = t14 * t6;
    let t16 = t12 + t15;
    fstp(w, 0x104, t16);
    let t17 = fild(w, 0x140);
    fstp(w, 0x108, t17);
    let t18 = fld(w, 0x48);
    let t19 = fld(w, 0x90);
    let t20 = t19 - t18;
    let t21 = t20 * t6;
    let t22 = t18 + t21;
    fstp(w, 0x118, t22);
    let t23 = fld(w, 0x4C);
    let t24 = fld(w, 0x94);
    let t25 = t24 - t23;
    let t26 = t25 * t6;
    let t27 = t23 + t26;
    fstp(w, 0x11C, t27);
    let t28 = fld(w, 0x50);
    let t29 = fld(w, 0x98);
    let t30 = t29 - t28;
    let t31 = t30 * t6;
    let t32 = t28 + t31;
    fstp(w, 0x120, t32);
    let t33 = fld(w, 0x54);
    let t34 = fld(w, 0x9C);
    let t35 = t34 - t33;
    let t36 = t35 * t6;
    let t37 = t33 + t36;
    fstp(w, 0x124, t37);
    let t38 = fild(w, 0xD0);
    let t39 = fild(w, 0xE4);
    let t40 = t39 - t38;
    let t41 = t40 * t6;
    let t42 = t38 + t41;
    fistp(w, 0x130, t42);
    let t43 = fild(w, 0xD4);
    let t44 = fild(w, 0xE8);
    let t45 = t44 - t43;
    let t46 = t45 * t6;
    let t47 = t43 + t46;
    fistp(w, 0x134, t47);
    let t48 = fild(w, 0xD8);
    let t49 = fild(w, 0xEC);
    let t50 = t49 - t48;
    let t51 = t50 * t6;
    let t52 = t48 + t51;
    fistp(w, 0x138, t52);
    let t53 = fild(w, 0xDC);
    let t54 = fild(w, 0xF0);
    let t55 = t54 - t53;
    let t56 = t55 * t6;
    let t57 = t53 + t56;
    fistp(w, 0x128, t57);
    let t58 = fild(w, 0xE0);
    let t59 = fild(w, 0xF4);
    let t60 = t59 - t58;
    let t61 = t60 * t6;
    let t62 = t58 + t61;
    fistp(w, 0x12C, t62);
    fstp(w, 0x13C, t6);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, Rng, differential};

    /// The original's instructions for `cut_at_x`.
    unsafe fn cut_x_ref(w: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
                "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                "fld dword ptr [{w} + 0xF8]", // fld SceneDraw.clip_work[0]
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                "fld dword ptr [{w} + 0xC]", // fld dword ptr [esi+0Ch]
                "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                ".byte 0xD9, 0xE0", // fchs
                ".byte 0xD8, 0xC2", // fadd st, st(2)
                "fld dword ptr [{w} + 0x2C]", // fld dword ptr [ebp+0Ch]
                "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                ".byte 0xD8, 0xC2", // fadd st, st(2)
                ".byte 0xD9, 0xCE", // fxch st(6)
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fld dword ptr [{w} + 0xF8]", // fld SceneDraw.clip_work[0]
                "fld dword ptr [{w} + 0x2C]", // fld dword ptr [ebp+0Ch]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                ".byte 0xD8, 0xF3", // fdiv st, st(3)
                ".byte 0xD9, 0xCB", // fxch st(3)
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fld dword ptr [{w} + 0x30]", // fld dword ptr [ebp+10h]
                "fld dword ptr [{w} + 0x10]", // fld dword ptr [esi+10h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x100]", // fstp SceneDraw.clip_work[2]
                "fld dword ptr [{w} + 0x34]", // fld dword ptr [ebp+14h]
                "fld dword ptr [{w} + 0x14]", // fld dword ptr [esi+14h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x104]", // fstp SceneDraw.clip_work[3]
                "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
                "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x108]", // fstp SceneDraw.clip_work[4]
                "fild dword ptr [{w} + 0x20]", // fild dword ptr [ebp+0]
                "fild dword ptr [{w} + 0x0]", // fild dword ptr [esi]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x10C]", // fistp SceneDraw.clip_work[5]
                "fild dword ptr [{w} + 0x24]", // fild dword ptr [ebp+4]
                "fild dword ptr [{w} + 0x4]", // fild dword ptr [esi+4]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x110]", // fistp SceneDraw.clip_work[6]
                "fild dword ptr [{w} + 0x28]", // fild dword ptr [ebp+8]
                "fild dword ptr [{w} + 0x8]", // fild dword ptr [esi+8]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x114]", // fistp SceneDraw.clip_work[7]
                "fld dword ptr [{w} + 0x90]", // fld dword ptr [ebp+8]
                "fld dword ptr [{w} + 0x48]", // fld dword ptr [esi+8]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x118]", // fstp SceneDraw.clip_work[8]
                "fld dword ptr [{w} + 0x94]", // fld dword ptr [ebp+0Ch]
                "fld dword ptr [{w} + 0x4C]", // fld dword ptr [esi+0Ch]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x11C]", // fstp SceneDraw.clip_work[9]
                "fld dword ptr [{w} + 0x98]", // fld dword ptr [ebp+10h]
                "fld dword ptr [{w} + 0x50]", // fld dword ptr [esi+10h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x120]", // fstp SceneDraw.clip_work[10]
                "fld dword ptr [{w} + 0x9C]", // fld dword ptr [ebp+14h]
                "fld dword ptr [{w} + 0x54]", // fld dword ptr [esi+14h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x124]", // fstp SceneDraw.clip_work[11]
                "fild dword ptr [{w} + 0xE4]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xD0]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x130]", // fistp SceneDraw.clip_out_words[2]
                "fild dword ptr [{w} + 0xE8]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xD4]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x134]", // fistp SceneDraw.clip_out_words[3]
                "fild dword ptr [{w} + 0xEC]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xD8]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x138]", // fistp SceneDraw.clip_out_words[4]
                "fild dword ptr [{w} + 0xF0]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xDC]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x128]", // fistp SceneDraw.clip_out_words[0]
                "fild dword ptr [{w} + 0xF4]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xE0]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x12C]", // fistp SceneDraw.clip_out_words[1]
                "fistp dword ptr [{w} + 0x13C]", // fistp SceneDraw.scratch
                "fld dword ptr [{w} + 0x2C]", // fld dword ptr [ebp+0Ch]
                "fld dword ptr [{w} + 0xC]", // fld dword ptr [esi+0Ch]
                ".byte 0xDC, 0xE9", // fsub st(1), st
                "fld dword ptr [{w} + 0xF8]", // fld SceneDraw.clip_work[0]
                ".byte 0xDE, 0xE9", // fsubp st(1), st
                "fld dword ptr [{w} + 0x3C]", // fld dword ptr [ebp+1Ch]
                "fld dword ptr [{w} + 0x1C]", // fld dword ptr [esi+1Ch]
                ".byte 0xDC, 0xE9", // fsub st(1), st
                ".byte 0xD9, 0xCA", // fxch st(2)
                ".byte 0xD9, 0xE0", // fchs
                ".byte 0xDE, 0xC9", // fmulp st(1), st
                ".byte 0xD9, 0xCA", // fxch st(2)
                ".byte 0xDE, 0xFA", // fdivp st(2), st
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0xFC]", // fstp SceneDraw.clip_work[1]
                w = in(reg) w,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `cut_at_y`.
    unsafe fn cut_y_ref(w: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
                "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                "fld dword ptr [{w} + 0xFC]", // fld SceneDraw.clip_work[1]
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                "fld dword ptr [{w} + 0x1C]", // fld dword ptr [esi+1Ch]
                "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                ".byte 0xD9, 0xE0", // fchs
                ".byte 0xD8, 0xC2", // fadd st, st(2)
                "fld dword ptr [{w} + 0x3C]", // fld dword ptr [ebp+1Ch]
                "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                ".byte 0xD8, 0xC2", // fadd st, st(2)
                ".byte 0xD9, 0xCE", // fxch st(6)
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fld dword ptr [{w} + 0xFC]", // fld SceneDraw.clip_work[1]
                "fld dword ptr [{w} + 0x3C]", // fld dword ptr [ebp+1Ch]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                ".byte 0xD8, 0xF3", // fdiv st, st(3)
                ".byte 0xD9, 0xCB", // fxch st(3)
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                "fld dword ptr [{w} + 0x30]", // fld dword ptr [ebp+10h]
                "fld dword ptr [{w} + 0x10]", // fld dword ptr [esi+10h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x100]", // fstp SceneDraw.clip_work[2]
                "fld dword ptr [{w} + 0x34]", // fld dword ptr [ebp+14h]
                "fld dword ptr [{w} + 0x14]", // fld dword ptr [esi+14h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x104]", // fstp SceneDraw.clip_work[3]
                "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
                "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x108]", // fstp SceneDraw.clip_work[4]
                "fild dword ptr [{w} + 0x20]", // fild dword ptr [ebp+0]
                "fild dword ptr [{w} + 0x0]", // fild dword ptr [esi]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x10C]", // fistp SceneDraw.clip_work[5]
                "fild dword ptr [{w} + 0x24]", // fild dword ptr [ebp+4]
                "fild dword ptr [{w} + 0x4]", // fild dword ptr [esi+4]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x110]", // fistp SceneDraw.clip_work[6]
                "fild dword ptr [{w} + 0x28]", // fild dword ptr [ebp+8]
                "fild dword ptr [{w} + 0x8]", // fild dword ptr [esi+8]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x114]", // fistp SceneDraw.clip_work[7]
                "fld dword ptr [{w} + 0x90]", // fld dword ptr [ebp+8]
                "fld dword ptr [{w} + 0x48]", // fld dword ptr [esi+8]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x118]", // fstp SceneDraw.clip_work[8]
                "fld dword ptr [{w} + 0x94]", // fld dword ptr [ebp+0Ch]
                "fld dword ptr [{w} + 0x4C]", // fld dword ptr [esi+0Ch]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x11C]", // fstp SceneDraw.clip_work[9]
                "fld dword ptr [{w} + 0x98]", // fld dword ptr [ebp+10h]
                "fld dword ptr [{w} + 0x50]", // fld dword ptr [esi+10h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x120]", // fstp SceneDraw.clip_work[10]
                "fld dword ptr [{w} + 0x9C]", // fld dword ptr [ebp+14h]
                "fld dword ptr [{w} + 0x54]", // fld dword ptr [esi+14h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x124]", // fstp SceneDraw.clip_work[11]
                "fild dword ptr [{w} + 0xE4]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xD0]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x130]", // fistp SceneDraw.clip_out_words[2]
                "fild dword ptr [{w} + 0xE8]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xD4]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x134]", // fistp SceneDraw.clip_out_words[3]
                "fild dword ptr [{w} + 0xEC]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xD8]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x138]", // fistp SceneDraw.clip_out_words[4]
                "fild dword ptr [{w} + 0xF0]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xDC]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x128]", // fistp SceneDraw.clip_out_words[0]
                "fild dword ptr [{w} + 0xF4]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xE0]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x12C]", // fistp SceneDraw.clip_out_words[1]
                "fistp dword ptr [{w} + 0x13C]", // fistp SceneDraw.scratch
                "fld dword ptr [{w} + 0x3C]", // fld dword ptr [ebp+1Ch]
                "fld dword ptr [{w} + 0x1C]", // fld dword ptr [esi+1Ch]
                ".byte 0xDC, 0xE9", // fsub st(1), st
                "fld dword ptr [{w} + 0xFC]", // fld SceneDraw.clip_work[1]
                ".byte 0xDE, 0xE9", // fsubp st(1), st
                "fld dword ptr [{w} + 0x2C]", // fld dword ptr [ebp+0Ch]
                "fld dword ptr [{w} + 0xC]", // fld dword ptr [esi+0Ch]
                ".byte 0xDC, 0xE9", // fsub st(1), st
                ".byte 0xD9, 0xCA", // fxch st(2)
                ".byte 0xD9, 0xE0", // fchs
                ".byte 0xDE, 0xC9", // fmulp st(1), st
                ".byte 0xD9, 0xCA", // fxch st(2)
                ".byte 0xDE, 0xFA", // fdivp st(2), st
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0xF8]", // fstp SceneDraw.clip_work[0]
                w = in(reg) w,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `cut_at_depth`.
    unsafe fn cut_depth_ref(w: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{w} + 0x140]", // fild SceneDraw.clip_plane
                "fsub dword ptr [{w} + 0x18]", // fsub dword ptr [esi+18h]
                "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
                "fsub dword ptr [{w} + 0x18]", // fsub dword ptr [esi+18h]
                ".byte 0xDE, 0xF9", // fdivp st(1), st
                "fst dword ptr [{w} + 0x13C]", // fst SceneDraw.scratch
                "fld dword ptr [{w} + 0x10]", // fld dword ptr [esi+10h]
                "fld dword ptr [{w} + 0x30]", // fld dword ptr [ebp+10h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x100]", // fstp SceneDraw.clip_work[2]
                "fld dword ptr [{w} + 0x14]", // fld dword ptr [esi+14h]
                "fld dword ptr [{w} + 0x34]", // fld dword ptr [ebp+14h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x104]", // fstp SceneDraw.clip_work[3]
                "fild dword ptr [{w} + 0x140]", // fild SceneDraw.clip_plane
                "fstp dword ptr [{w} + 0x108]", // fstp SceneDraw.clip_work[4]
                "fld dword ptr [{w} + 0x48]", // fld dword ptr [esi+8]
                "fld dword ptr [{w} + 0x90]", // fld dword ptr [ebp+8]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x118]", // fstp SceneDraw.clip_work[8]
                "fld dword ptr [{w} + 0x4C]", // fld dword ptr [esi+0Ch]
                "fld dword ptr [{w} + 0x94]", // fld dword ptr [ebp+0Ch]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x11C]", // fstp SceneDraw.clip_work[9]
                "fld dword ptr [{w} + 0x50]", // fld dword ptr [esi+10h]
                "fld dword ptr [{w} + 0x98]", // fld dword ptr [ebp+10h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x120]", // fstp SceneDraw.clip_work[10]
                "fld dword ptr [{w} + 0x54]", // fld dword ptr [esi+14h]
                "fld dword ptr [{w} + 0x9C]", // fld dword ptr [ebp+14h]
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{w} + 0x124]", // fstp SceneDraw.clip_work[11]
                "fild dword ptr [{w} + 0xD0]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xE4]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x130]", // fistp SceneDraw.clip_out_words[2]
                "fild dword ptr [{w} + 0xD4]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xE8]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x134]", // fistp SceneDraw.clip_out_words[3]
                "fild dword ptr [{w} + 0xD8]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xEC]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x138]", // fistp SceneDraw.clip_out_words[4]
                "fild dword ptr [{w} + 0xDC]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xF0]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x128]", // fistp SceneDraw.clip_out_words[0]
                "fild dword ptr [{w} + 0xE0]", // fild SceneDraw.scratch
                "fild dword ptr [{w} + 0xF4]", // fild SceneDraw.scratch
                ".byte 0xD8, 0xE1", // fsub st, st(1)
                ".byte 0xD8, 0xCA", // fmul st, st(2)
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fistp dword ptr [{w} + 0x12C]", // fistp SceneDraw.clip_out_words[1]
                "fstp dword ptr [{w} + 0x13C]", // fstp SceneDraw.scratch
                w = in(reg) w,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// A block: the whole numbers where the original reads them with `fild` (the records'
    /// first three, the entries' five bytes and the plane), floats elsewhere.
    fn fill(rng: &mut Rng, m: &mut [u32; 0x51]) {
        let moderate = rng.below(2) == 0;
        for d in m.iter_mut() {
            *d = if moderate { rng.moderate() } else { rng.fl() };
        }
        for k in [0, 1, 2, 8, 9, 10, 80].into_iter().chain(52..62) {
            m[k] = if rng.below(2) == 0 {
                rng.small() as u32
            } else {
                rng.int()
            };
        }
    }

    #[test]
    fn cut_at_x_matches_the_assembly() {
        differential::<0x51>(
            "cut_at_x",
            4000,
            30,
            [Kind::Float; 0x51],
            fill,
            |base| unsafe { cut_x_ref(base) },
            |w| cut_axis(w, X_AXIS),
        );
    }

    #[test]
    fn cut_at_y_matches_the_assembly() {
        differential::<0x51>(
            "cut_at_y",
            4000,
            31,
            [Kind::Float; 0x51],
            fill,
            |base| unsafe { cut_y_ref(base) },
            |w| cut_axis(w, Y_AXIS),
        );
    }

    #[test]
    fn cut_at_depth_matches_the_assembly() {
        differential::<0x51>(
            "cut_at_depth",
            4000,
            32,
            [Kind::Float; 0x51],
            fill,
            |base| unsafe { cut_depth_ref(base) },
            cut_depth,
        );
    }
}
