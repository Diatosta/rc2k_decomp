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
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::windscreen_drops_frame` (0x4BC0BA). The x87 runs are plain Rust on the x87
//! model (`math::x87_float`), in the original's order, over the game's variables (the three values
//! it keeps on the x87 stack across the two calls kept as 80 bits); the calls their Rust versions.
//! Refuses null settings, before anything.

use crate::effects::adjugate3::adjugate3;
use crate::effects::setup::MarksClock;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::{Axis, extended, turn_matrix3};
use crate::session::{Bytes, Float};

/// The original's 0x4BCC3D .. 0x4BCD03: the view's move (the settings' +0x26AC .. less the kept `flt_8161D6` .., times `flt_764FE0`) into `flt_8161EE` .., less the last move into `flt_8161FA` .., plus the last move times `flt_82E326` ...
fn view_moved(
    drops: &WindscreenDrops,
    m: &EffectsSettings,
    effects: &crate::effects::owner::Effects,
) {
    view_moved_x87(drops, m, &effects.setup.clock);
}

/// The original's 0x4BCE6A .. 0x4BCEB2: the view's move kept in `flt_8161E2` ...
fn move_kept(
    drops: &WindscreenDrops,
    m: &EffectsSettings,
    effects: &crate::effects::owner::Effects,
) {
    move_kept_x87(drops, m, &effects.setup.clock);
}

pub fn windscreen_drops_drift(
    frame: &Frame,
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let m = crate::effects::setup::effects_settings()?;
    let marks = crate::effects::setup::effects_settings()?;
    for n in 0..3 {
        drops.tilt.point[n].set_bits(0);
    }
    let first: u32 = drops.hex.started.get();
    drops.hex.started.set(1);
    if first != 0 {
        let second: u32 = drops.hex.moved.get();
        drops.hex.moved.set(1);
        if second != 0 {
            view_moved(drops, m, effects);
            let view = &frame.matrices.view_place;
            for col in 0..3usize {
                let v = [view[col].bits(), view[4 + col].bits(), view[8 + col].bits()];
                for (k, x) in v.into_iter().enumerate() {
                    drops.hex.turn[3 * col + k].set_bits(x);
                }
            }
            adjugate3(&drops.hex.turn, &drops.hex.adjugate);
            // The two temporaries `[esp]` held and the three 80-bit values kept across the two calls.
            let temps = [Float::from_bits(0), Float::from_bits(0)];
            let kept = Bytes::new([0u8; 30]);
            drift_turned_x87(drops, &drops.hex.adjugate, &temps, &kept);
            turn_matrix3(Axis::X, &drops.hex.turn, extended(drops.hex.angle.bits()))?;
            drift_back_x87(drops, &drops.hex.turn, &temps, &kept);
        }
        move_kept(drops, m, effects);
    }
    for (k, v) in [
        marks.listener_car_place[0].bits(),
        marks.listener_car_place[1].bits(),
        marks.listener_car_place[2].bits(),
    ]
    .into_iter()
    .enumerate()
    {
        drops.hex.kept_place[k].set_bits(v);
    }
    Ok(())
}

use crate::effects::plain::{ld_cell, ld80_at, st_cell, st80_at};
use crate::math::x87_float as bh;

/// `view_moved`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
fn view_moved_x87(drops: &WindscreenDrops, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&m.listener_car_place[0]);
    let t1 = t0 - ld_cell(&drops.hex.kept_place[0]);
    let t2 = t1 * crate::math::x87_float::ld(c.reciprocal.bits());
    let t3 = ld_cell(&m.listener_car_place[1]);
    let t4 = t3 - ld_cell(&drops.hex.kept_place[1]);
    let t5 = t4 * crate::math::x87_float::ld(c.reciprocal.bits());
    let t6 = ld_cell(&m.listener_car_place[2]);
    let t7 = t6 - ld_cell(&drops.hex.kept_place[2]);
    let t8 = t7 * crate::math::x87_float::ld(c.reciprocal.bits());
    st_cell(&drops.hex.view_move[2], t8);
    st_cell(&drops.hex.view_move[1], t5);
    st_cell(&drops.hex.view_move[0], t2);
    let t9 = ld_cell(&drops.hex.view_move[0]);
    let t10 = t9 - ld_cell(&drops.hex.kept_move[0]);
    let t11 = ld_cell(&drops.hex.view_move[1]);
    let t12 = t11 - ld_cell(&drops.hex.kept_move[1]);
    let t13 = ld_cell(&drops.hex.view_move[2]);
    let t14 = t13 - ld_cell(&drops.hex.kept_move[2]);
    st_cell(&drops.hex.move_change[2], t14);
    st_cell(&drops.hex.move_change[1], t12);
    st_cell(&drops.hex.move_change[0], t10);
    let t15 = ld_cell(&drops.hex.kept_move[0]);
    let t16 = t15 * ld_cell(&drops.hex.decay[0]);
    let t17 = t16 + ld_cell(&drops.hex.move_change[0]);
    st_cell(&drops.hex.move_change[0], t17);
    let t18 = ld_cell(&drops.hex.kept_move[1]);
    let t19 = t18 * ld_cell(&drops.hex.decay[1]);
    let t20 = t19 + ld_cell(&drops.hex.move_change[1]);
    st_cell(&drops.hex.move_change[1], t20);
    let t21 = ld_cell(&drops.hex.kept_move[2]);
    let t22 = t21 * ld_cell(&drops.hex.decay[2]);
    let t23 = t22 + ld_cell(&drops.hex.move_change[2]);
    st_cell(&drops.hex.move_change[2], t23);
}

/// `drift_turned`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
fn drift_turned_x87(drops: &WindscreenDrops, m: &[Float; 9], t: &[Float; 2], l: &Bytes<30>) {
    let t0 = ld_cell(&drops.hex.move_change[2]);
    let t1 = ld_cell(&drops.hex.move_change[1]);
    let t2 = ld_cell(&drops.hex.move_change[0]);
    let t3 = t2 * ld_cell(&m[0]);
    let t4 = t1 * ld_cell(&m[1]);
    let t5 = t0 * ld_cell(&m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st_cell(&t[0], t7);
    let t8 = t2 * ld_cell(&m[3]);
    let t9 = t1 * ld_cell(&m[4]);
    let t10 = t0 * ld_cell(&m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st_cell(&t[1], t12);
    let t13 = t2 * ld_cell(&m[6]);
    let t14 = t1 * ld_cell(&m[7]);
    let t15 = t0 * ld_cell(&m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld_cell(&t[1]);
    let t19 = ld_cell(&t[0]);
    st80_at(l, 0, t19);
    st80_at(l, 10, t18);
    st80_at(l, 20, t17);
}

/// `drift_back`'s x87 block as plain Rust on the x87 model (40 instructions), same order and memory places.
fn drift_back_x87(drops: &WindscreenDrops, m: &[Float; 9], t: &[Float; 2], l: &Bytes<30>) {
    let t0 = bh::from_f80(ld80_at(l, 20));
    let t1 = bh::from_f80(ld80_at(l, 10));
    let t2 = bh::from_f80(ld80_at(l, 0));
    let t3 = t2 * ld_cell(&m[0]);
    let t4 = t1 * ld_cell(&m[1]);
    let t5 = t0 * ld_cell(&m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st_cell(&t[0], t7);
    let t8 = t2 * ld_cell(&m[3]);
    let t9 = t1 * ld_cell(&m[4]);
    let t10 = t0 * ld_cell(&m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st_cell(&t[1], t12);
    let t13 = t2 * ld_cell(&m[6]);
    let t14 = t1 * ld_cell(&m[7]);
    let t15 = t0 * ld_cell(&m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld_cell(&t[1]);
    let t19 = ld_cell(&t[0]);
    let t20 = -t19;
    st_cell(&drops.hex.drift[0], t20);
    let t21 = t20 * ld_cell(&drops.hex.result_k[0]);
    drops.tilt.point[0].set_bits(bh::st(t21));
    st_cell(&drops.hex.drift[1], t18);
    let t22 = t18 * ld_cell(&drops.hex.result_k[1]);
    drops.tilt.point[1].set_bits(bh::st(t22));
    st_cell(&drops.hex.drift[2], t17);
    let t23 = t17 * ld_cell(&drops.hex.result_k[2]);
    drops.tilt.point[2].set_bits(bh::st(t23));
}

/// `move_kept`'s x87 block as plain Rust on the x87 model (12 instructions), same order and memory places.
fn move_kept_x87(drops: &WindscreenDrops, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&m.listener_car_place[0]);
    let t1 = t0 - ld_cell(&drops.hex.kept_place[0]);
    let t2 = t1 * crate::math::x87_float::ld(c.reciprocal.bits());
    let t3 = ld_cell(&m.listener_car_place[1]);
    let t4 = t3 - ld_cell(&drops.hex.kept_place[1]);
    let t5 = t4 * crate::math::x87_float::ld(c.reciprocal.bits());
    let t6 = ld_cell(&m.listener_car_place[2]);
    let t7 = t6 - ld_cell(&drops.hex.kept_place[2]);
    let t8 = t7 * crate::math::x87_float::ld(c.reciprocal.bits());
    st_cell(&drops.hex.kept_move[2], t8);
    st_cell(&drops.hex.kept_move[1], t5);
    st_cell(&drops.hex.kept_move[0], t2);
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
            );
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
            );
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
            );
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
            );
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::with_settings;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::tilt_shape::ShapeTilt;
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::windscreen_drop_vars::WindscreenDropVars;
    use crate::session::{Bytes, Float};

    /// Runs `f` with `HEX` holding the buffer's bytes (`typed_sync`).
    fn hex<R>(mk: usize, f: impl FnOnce() -> R) -> R {
        with_overlays(
            mk,
            &[overlay(
                &crate::effects::owner::EFFECTS.windscreen_drops.hex,
                WindscreenDropVars::START,
            )],
            f,
        )
    }

    /// The `N` floats at the test address `at`.
    fn floats<const N: usize>(at: usize) -> [Float; N] {
        std::array::from_fn(|i| {
            let word = unsafe {
                crate::pointers::host_at(at + 4 * i)
                    .cast::<u32>()
                    .read_unaligned()
            };
            Float::from_bits(word)
        })
    }

    /// The floats written back to the test address `at`.
    fn put_floats(at: usize, cells: &[Float]) {
        for (i, c) in cells.iter().enumerate() {
            unsafe {
                crate::pointers::host_at(at + 4 * i)
                    .cast::<u32>()
                    .write_unaligned(c.bits());
            }
        }
    }

    /// The 30 bytes at the test address `at` as a buffer.
    fn kept(at: usize) -> Bytes<30> {
        Bytes::new(std::array::from_fn(|i| unsafe {
            crate::pointers::host_at(at + i).read()
        }))
    }

    fn put_kept(at: usize, kept: &Bytes<30>) {
        for i in 0..30 {
            unsafe { crate::pointers::host_at(at + i).write(kept.get(i)) };
        }
    }

    #[test]
    fn view_moved_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xA0,
                    hi: 0xC9402,
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
                    lo: 0,
                    hi: 0x2768,
                    reads: &[(0x26AC, F32), (0x26B0, F32), (0x26B4, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::view_moved_x87(b[0], b[1]),
            |b| {
                hex(b[0], || {
                    with_settings(b[1], |m| {
                        super::view_moved_x87(
                            &crate::effects::owner::EFFECTS.windscreen_drops,
                            m,
                            crate::effects::setup::clock_at(b[0]),
                        );
                    });
                });
            },
        );
    }

    #[test]
    fn drift_turned_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xB1252,
                    hi: 0xC9402,
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
            |b| {
                hex(b[0], || {
                    let (m, temps, l) = (floats::<9>(b[1]), floats::<2>(b[2]), kept(b[3]));
                    super::drift_turned_x87(
                        &crate::effects::owner::EFFECTS.windscreen_drops,
                        &m,
                        &temps,
                        &l,
                    );
                    put_floats(b[2], &temps);
                    put_kept(b[3], &l);
                });
            },
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
                with_overlays(
                    b[0],
                    &[
                        overlay(
                            &crate::effects::owner::EFFECTS.windscreen_drops.tilt,
                            ShapeTilt::START,
                        ),
                        overlay(
                            &crate::effects::owner::EFFECTS.windscreen_drops.hex,
                            WindscreenDropVars::START,
                        ),
                    ],
                    || {
                        let (m, temps, l) = (floats::<9>(b[1]), floats::<2>(b[2]), kept(b[3]));
                        super::drift_back_x87(
                            &crate::effects::owner::EFFECTS.windscreen_drops,
                            &m,
                            &temps,
                            &l,
                        );
                        put_floats(b[2], &temps);
                    },
                );
            },
        );
    }

    #[test]
    fn move_kept_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xA0,
                    hi: 0xC9402,
                    reads: &[(0xA0, F32), (0xB1296, F32), (0xB129A, F32), (0xB129E, F32)],
                    writes: &[(0xB12A2, F32), (0xB12A6, F32), (0xB12AA, F32)],
                },
                Op::Base {
                    lo: 0,
                    hi: 0x2768,
                    reads: &[(0x26AC, F32), (0x26B0, F32), (0x26B4, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::move_kept_x87(b[0], b[1]),
            |b| {
                hex(b[0], || {
                    with_settings(b[1], |m| {
                        super::move_kept_x87(
                            &crate::effects::owner::EFFECTS.windscreen_drops,
                            m,
                            crate::effects::setup::clock_at(b[0]),
                        );
                    });
                });
            },
        );
    }
}
