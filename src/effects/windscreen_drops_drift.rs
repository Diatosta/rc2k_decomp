//! `effects::windscreen_drops_drift` (0x4BCBE9): the windscreen drops' drift from the view's moves.
//! `flt_811BDF` .. 0; unless this is the first time (`dword_8161AE`, then 1): unless the
//! second (`dword_8161B2`, then 1), the view's move (the settings' +0x26AC .. less the kept
//! place, times `flt_764FE0`) less the last move (plus the last times `flt_82E326` ..),
//! turned by the adjugate of the view's turn (`flt_5EA3CC`'s, copied by columns into
//! `dword_82E2C6`, `effects::adjugate3` into `unk_82E2F6`), the matrix made a turn about x by
//! `flt_82E332` (`scene::turn_matrix3_x`) and the move turned back by it, negated, into
//! `flt_81621E` .. and times `flt_82E336` .. into `flt_811BDF` ..; then the move kept
//! (`flt_8161E2` ..); and the place kept (`flt_8161D6` ..).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BC0B1` (0x4BC0BA). The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables (the three values it keeps on the x87 stack across the two calls kept
//! as 80 bits); the calls their Rust versions. Refuses null settings, before anything.

use crate::effects::adjugate3::adjugate3;
use crate::effects::setup::{CLOCK, MarksClock};
use crate::effects::tilt_shape::SHAPE_TILT;
use crate::effects::windscreen_drop_vars::HEX;
use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::race::matrices_store::MATRICES;
use crate::scene::matrix3::{Axis, extended, turn_matrix3};

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a game field")
}

/// The original's 0x4BCC3D .. 0x4BCD03: the view's move (the settings' +0x26AC .. less the kept `flt_8161D6` .., times `flt_764FE0`) into `flt_8161EE` .., less the last move into `flt_8161FA` .., plus the last move times `flt_82E326` ...
fn view_moved(s: u32) {
    view_moved_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        s as usize,
        &CLOCK,
    )
}

/// The original's 0x4BCD69 .. 0x4BCDCD: that move turned by the adjugate at m (`[esp]` temporaries at t), the three kept 80 bits at l.
fn drift_turned(m: u32, t: u32, l: u32) {
    drift_turned_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        m as usize,
        t as usize,
        l as usize,
    )
}

/// The original's 0x4BCDE6 .. 0x4BCE6A: the three (from l) turned back by the matrix at m, negated, into `flt_81621E` .. and times `flt_82E336` .. into `flt_811BDF` ...
fn drift_back(m: u32, t: u32, l: u32) {
    drift_back_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        m as usize,
        t as usize,
        l as usize,
    )
}

/// The original's 0x4BCE6A .. 0x4BCEB2: the view's move kept in `flt_8161E2` ...
fn move_kept(s: u32) {
    move_kept_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        s as usize,
        &CLOCK,
    )
}

/// The Rust version (see the header).
pub fn windscreen_drops_drift() -> Result<(), Refused> {
    let s: u32 = crate::effects::setup::SETTINGS.pointer.get();
    if s == 0 {
        return Err(BadInput("no marks settings").into());
    }
    for n in 0..3 {
        SHAPE_TILT.point[n].set_bits(0);
    }
    let first: u32 = HEX.started.get();
    HEX.started.set(1);
    if first != 0 {
        let second: u32 = HEX.moved.get();
        HEX.moved.set(1);
        if second != 0 {
            view_moved(s);
            let view = MATRICES.view_place[0].addr();
            for col in 0..3u32 {
                let v = [
                    get(view + 4 * col)?,
                    get(view + 0x10 + 4 * col)?,
                    get(view + 0x20 + 4 * col)?,
                ];
                for (k, x) in v.into_iter().enumerate() {
                    HEX.turn[3 * col as usize + k].set_bits(x);
                }
            }
            let turn = crate::pointers::game_address(HEX.turn.as_ptr());
            let adj = crate::pointers::game_address(HEX.adjugate.as_ptr());
            adjugate3(turn, adj)?;
            let mut temps = [0u32; 2];
            let mut kept = [0u8; 30];
            let (t, l) = (
                crate::pointers::game_address(temps.as_mut_ptr()),
                crate::pointers::game_address(kept.as_mut_ptr()),
            );
            drift_turned(adj, t, l);
            turn_matrix3(Axis::X, &HEX.turn, extended(HEX.angle.bits()))?;
            drift_back(turn, t, l);
        }
        move_kept(s);
    }
    for (k, v) in [get(s + 0x26AC)?, get(s + 0x26B0)?, get(s + 0x26B4)?]
        .into_iter()
        .enumerate()
    {
        HEX.kept_place[k].set_bits(v);
    }
    Ok(())
}

use crate::effects::plain::{ld, ld80, st, st80};
use crate::math::x87_float as bh;

/// `view_moved`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
fn view_moved_x87(hx: usize, s: usize, c: &MarksClock) {
    let t0 = ld(s, 0x26AC);
    let t1 = t0 - ld(hx, 0xB1296);
    let t2 = t1 * crate::math::x87_float::ld(c.reciprocal.bits());
    let t3 = ld(s, 0x26B0);
    let t4 = t3 - ld(hx, 0xB129A);
    let t5 = t4 * crate::math::x87_float::ld(c.reciprocal.bits());
    let t6 = ld(s, 0x26B4);
    let t7 = t6 - ld(hx, 0xB129E);
    let t8 = t7 * crate::math::x87_float::ld(c.reciprocal.bits());
    st(hx, 0xB12B6, t8);
    st(hx, 0xB12B2, t5);
    st(hx, 0xB12AE, t2);
    let t9 = ld(hx, 0xB12AE);
    let t10 = t9 - ld(hx, 0xB12A2);
    let t11 = ld(hx, 0xB12B2);
    let t12 = t11 - ld(hx, 0xB12A6);
    let t13 = ld(hx, 0xB12B6);
    let t14 = t13 - ld(hx, 0xB12AA);
    st(hx, 0xB12C2, t14);
    st(hx, 0xB12BE, t12);
    st(hx, 0xB12BA, t10);
    let t15 = ld(hx, 0xB12A2);
    let t16 = t15 * ld(hx, 0xC93E6);
    let t17 = t16 + ld(hx, 0xB12BA);
    st(hx, 0xB12BA, t17);
    let t18 = ld(hx, 0xB12A6);
    let t19 = t18 * ld(hx, 0xC93EA);
    let t20 = t19 + ld(hx, 0xB12BE);
    st(hx, 0xB12BE, t20);
    let t21 = ld(hx, 0xB12AA);
    let t22 = t21 * ld(hx, 0xC93EE);
    let t23 = t22 + ld(hx, 0xB12C2);
    st(hx, 0xB12C2, t23);
}

/// `drift_turned`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
fn drift_turned_x87(hx: usize, m: usize, t: usize, l: usize) {
    let t0 = ld(hx, 0xB12C2);
    let t1 = ld(hx, 0xB12BE);
    let t2 = ld(hx, 0xB12BA);
    let t3 = t2 * ld(m, 0);
    let t4 = t1 * ld(m, 0x4);
    let t5 = t0 * ld(m, 0x8);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st(t, 0, t7);
    let t8 = t2 * ld(m, 0xC);
    let t9 = t1 * ld(m, 0x10);
    let t10 = t0 * ld(m, 0x14);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st(t, 0x4, t12);
    let t13 = t2 * ld(m, 0x18);
    let t14 = t1 * ld(m, 0x1C);
    let t15 = t0 * ld(m, 0x20);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld(t, 0x4);
    let t19 = ld(t, 0);
    st80(l, 0, t19);
    st80(l, 0xA, t18);
    st80(l, 0x14, t17);
}

/// `drift_back`'s x87 block as plain Rust on the x87 model (40 instructions), same order and memory places.
fn drift_back_x87(hx: usize, m: usize, t: usize, l: usize) {
    let t0 = bh::from_f80(ld80(l, 0x14));
    let t1 = bh::from_f80(ld80(l, 0xA));
    let t2 = bh::from_f80(ld80(l, 0));
    let t3 = t2 * ld(m, 0);
    let t4 = t1 * ld(m, 0x4);
    let t5 = t0 * ld(m, 0x8);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st(t, 0, t7);
    let t8 = t2 * ld(m, 0xC);
    let t9 = t1 * ld(m, 0x10);
    let t10 = t0 * ld(m, 0x14);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st(t, 0x4, t12);
    let t13 = t2 * ld(m, 0x18);
    let t14 = t1 * ld(m, 0x1C);
    let t15 = t0 * ld(m, 0x20);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld(t, 0x4);
    let t19 = ld(t, 0);
    let t20 = -t19;
    st(hx, 0xB12DE, t20);
    let t21 = t20 * ld(hx, 0xC93F6);
    SHAPE_TILT.point[0].set_bits(bh::st(t21));
    st(hx, 0xB12E2, t18);
    let t22 = t18 * ld(hx, 0xC93FA);
    SHAPE_TILT.point[1].set_bits(bh::st(t22));
    st(hx, 0xB12E6, t17);
    let t23 = t17 * ld(hx, 0xC93FE);
    SHAPE_TILT.point[2].set_bits(bh::st(t23));
}

/// `move_kept`'s x87 block as plain Rust on the x87 model (12 instructions), same order and memory places.
fn move_kept_x87(hx: usize, s: usize, c: &MarksClock) {
    let t0 = ld(s, 0x26AC);
    let t1 = t0 - ld(hx, 0xB1296);
    let t2 = t1 * crate::math::x87_float::ld(c.reciprocal.bits());
    let t3 = ld(s, 0x26B0);
    let t4 = t3 - ld(hx, 0xB129A);
    let t5 = t4 * crate::math::x87_float::ld(c.reciprocal.bits());
    let t6 = ld(s, 0x26B4);
    let t7 = t6 - ld(hx, 0xB129E);
    let t8 = t7 * crate::math::x87_float::ld(c.reciprocal.bits());
    st(hx, 0xB12AA, t8);
    st(hx, 0xB12A6, t5);
    st(hx, 0xB12A2, t2);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn view_moved_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{s} + 0x26AC]",
            "fsub dword ptr [{mk} + 0xB1296]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fld dword ptr [{s} + 0x26B0]",
            "fsub dword ptr [{mk} + 0xB129A]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fld dword ptr [{s} + 0x26B4]",
            "fsub dword ptr [{mk} + 0xB129E]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fstp dword ptr [{mk} + 0xB12B6]",
            "fstp dword ptr [{mk} + 0xB12B2]",
            "fstp dword ptr [{mk} + 0xB12AE]",
            "fld dword ptr [{mk} + 0xB12AE]",
            "fsub dword ptr [{mk} + 0xB12A2]",
            "fld dword ptr [{mk} + 0xB12B2]",
            "fsub dword ptr [{mk} + 0xB12A6]",
            "fld dword ptr [{mk} + 0xB12B6]",
            "fsub dword ptr [{mk} + 0xB12AA]",
            "fstp dword ptr [{mk} + 0xB12C2]",
            "fstp dword ptr [{mk} + 0xB12BE]",
            "fstp dword ptr [{mk} + 0xB12BA]",
            "fld dword ptr [{mk} + 0xB12A2]",
            "fmul dword ptr [{mk} + 0xC93E6]",
            "fadd dword ptr [{mk} + 0xB12BA]",
            "fstp dword ptr [{mk} + 0xB12BA]",
            "fld dword ptr [{mk} + 0xB12A6]",
            "fmul dword ptr [{mk} + 0xC93EA]",
            "fadd dword ptr [{mk} + 0xB12BE]",
            "fstp dword ptr [{mk} + 0xB12BE]",
            "fld dword ptr [{mk} + 0xB12AA]",
            "fmul dword ptr [{mk} + 0xC93EE]",
            "fadd dword ptr [{mk} + 0xB12C2]",
            "fstp dword ptr [{mk} + 0xB12C2]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) fn drift_turned_x87(mk: usize, m: usize, t: usize, l: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0xB12C2]",
            "fld dword ptr [{mk} + 0xB12BE]",
            "fld dword ptr [{mk} + 0xB12BA]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp tbyte ptr [{l}]",
            "fstp tbyte ptr [{l} + 10]",
            "fstp tbyte ptr [{l} + 20]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            l = in(reg) l,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) fn drift_back_x87(mk: usize, m: usize, t: usize, l: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{l} + 20]",
            "fld tbyte ptr [{l} + 10]",
            "fld tbyte ptr [{l}]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xE0",
            "fst dword ptr [{mk} + 0xB12DE]",
            "fmul dword ptr [{mk} + 0xC93F6]",
            "fstp dword ptr [{mk} + 0xACC9F]",
            "fst dword ptr [{mk} + 0xB12E2]",
            "fmul dword ptr [{mk} + 0xC93FA]",
            "fstp dword ptr [{mk} + 0xACCA3]",
            "fst dword ptr [{mk} + 0xB12E6]",
            "fmul dword ptr [{mk} + 0xC93FE]",
            "fstp dword ptr [{mk} + 0xACCA7]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            l = in(reg) l,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) fn move_kept_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{s} + 0x26AC]",
            "fsub dword ptr [{mk} + 0xB1296]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fld dword ptr [{s} + 0x26B0]",
            "fsub dword ptr [{mk} + 0xB129A]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fld dword ptr [{s} + 0x26B4]",
            "fsub dword ptr [{mk} + 0xB129E]",
            "fmul dword ptr [{mk} + 0xA0]",
            "fstp dword ptr [{mk} + 0xB12AA]",
            "fstp dword ptr [{mk} + 0xB12A6]",
            "fstp dword ptr [{mk} + 0xB12A2]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::tilt_shape::{SHAPE_TILT, ShapeTilt};
    use crate::effects::typed_sync::{overlay, with_overlays};

    #[test]
    fn view_moved_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xA0,
                    hi: 0xC93F2,
                    reads: &[
                        (0xA0, F32),
                        (0xB1296, F32),
                        (0xB129A, F32),
                        (0xB129E, F32),
                        (0xB12A2, F32),
                        (0xB12A6, F32),
                        (0xB12AA, F32),
                        (0xB12AE, F32),
                        (0xB12B2, F32),
                        (0xB12B6, F32),
                        (0xB12BA, F32),
                        (0xB12BE, F32),
                        (0xB12C2, F32),
                        (0xC93E6, F32),
                        (0xC93EA, F32),
                        (0xC93EE, F32),
                    ],
                    writes: &[
                        (0xB12AE, F32),
                        (0xB12B2, F32),
                        (0xB12B6, F32),
                        (0xB12BA, F32),
                        (0xB12BE, F32),
                        (0xB12C2, F32),
                    ],
                },
                Op::Base {
                    lo: 0x26AC,
                    hi: 0x26B8,
                    reads: &[(0x26AC, F32), (0x26B0, F32), (0x26B4, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::view_moved_x87(b[0], b[1]),
            |b| super::view_moved_x87(b[0], b[1], crate::effects::setup::clock_at(b[0])),
        );
    }

    #[test]
    fn drift_turned_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xB12BA,
                    hi: 0xB12C6,
                    reads: &[(0xB12BA, F32), (0xB12BE, F32), (0xB12C2, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x1E,
                    reads: &[],
                    writes: &[(0x0, F80), (0xA, F80), (0x14, F80)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::drift_turned_x87(b[0], b[1], b[2], b[3]),
            |b| super::drift_turned_x87(b[0], b[1], b[2], b[3]),
        );
    }

    #[test]
    fn drift_back_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xACC83,
                    hi: 0xC9402,
                    reads: &[(0xC93F6, F32), (0xC93FA, F32), (0xC93FE, F32)],
                    writes: &[
                        (0xACC9F, F32),
                        (0xACCA3, F32),
                        (0xACCA7, F32),
                        (0xB12DE, F32),
                        (0xB12E2, F32),
                        (0xB12E6, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x1E,
                    reads: &[(0x0, F80), (0xA, F80), (0x14, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::drift_back_x87(b[0], b[1], b[2], b[3]),
            |b| {
                with_overlays(b[0], &[overlay(&SHAPE_TILT, ShapeTilt::START)], || {
                    super::drift_back_x87(b[0], b[1], b[2], b[3])
                })
            },
        );
    }

    #[test]
    fn move_kept_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xA0,
                    hi: 0xB12AE,
                    reads: &[(0xA0, F32), (0xB1296, F32), (0xB129A, F32), (0xB129E, F32)],
                    writes: &[(0xB12A2, F32), (0xB12A6, F32), (0xB12AA, F32)],
                },
                Op::Base {
                    lo: 0x26AC,
                    hi: 0x26B8,
                    reads: &[(0x26AC, F32), (0x26B0, F32), (0x26B4, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::move_kept_x87(b[0], b[1]),
            |b| super::move_kept_x87(b[0], b[1], crate::effects::setup::clock_at(b[0])),
        );
    }
}
