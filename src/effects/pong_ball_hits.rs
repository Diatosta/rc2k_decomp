//! `effects::pong_ball_hits` (0x4B8040): when the pong ball overlaps the bat (`effects::pong`'s
//! four compares): a hit (`dword_7687D0` plus 1), four more steps a frame
//! (`g_pongSteps`), the ball a step back, and its new way: from the bat's centre to
//! the ball's (`flt_7686DC` .. `flt_7686F8`; an across of exactly 0 bits less
//! `flt_7691D6`), made one long times `flt_76516C`, times `flt_7691DA` / `flt_7691DE`.
//!
//! Caller: `effects::move_pong_ball`. The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::cell_x87::CellX87;
use crate::effects::pong::{self, Pong, PongRules, PongState};
use crate::math::x87;

/// The original's 0x4B8105 .. 0x4B8189: the centres and their difference.
fn centres(pong: &Pong) {
    centres_x87(&pong.state, &pong.rules);
}

/// The original's 0x4B81A4 .. 0x4B81FE: the difference made one long, scaled.
fn new_way(pong: &Pong, effects: &crate::effects::owner::Effects) {
    new_way_x87(&pong.state, &pong.rules, effects.setup.floats.one.bits());
}

pub fn pong_ball_hits(pong: &Pong, effects: &crate::effects::owner::Effects) {
    let (over, _) = pong::overlap(pong);
    if !over {
        return;
    }
    let p = &pong.state;
    p.hits.set(p.hits.get().wrapping_add(1));
    p.steps.set(p.steps.get().wrapping_add(4));
    for n in 0..2 {
        p.ball[n].set(x87::sub(&p.ball[n], &p.ball_way[n]));
    }
    centres(pong);
    if p.difference[0].bits() == 0 {
        p.difference[0].set(x87::sub(&p.difference[0], &pong.rules.nudge));
    }
    new_way(pong, effects);
    p.ball_way[0].set_bits(p.difference[0].bits());
    p.ball_way[1].set_bits(p.difference[1].bits());
}

use crate::math::x87_float::ld;

/// `centres`'s x87 block as plain Rust on the x87 model (22 instructions), same order and memory places.
fn centres_x87(p: &PongState, r: &PongRules) {
    let t0 = p.ball_size[0].ld();
    let t1 = t0 * r.centre[0].ld();
    let t2 = t1 + p.ball[0].ld();
    p.ball_centre[0].st(t2);
    let t3 = p.ball_size[1].ld();
    let t4 = t3 * r.centre[1].ld();
    let t5 = t4 + p.ball[1].ld();
    p.ball_centre[1].st(t5);
    let t6 = p.bat_size[0].ld();
    let t7 = t6 * r.centre[2].ld();
    let t8 = t7 + p.bat[0].ld();
    p.bat_centre[0].st(t8);
    let t9 = p.bat_size[1].ld();
    let t10 = t9 * r.centre[3].ld();
    let t11 = t10 + p.bat[1].ld();
    p.bat_centre[1].st(t11);
    let t12 = p.ball_centre[0].ld();
    let t13 = t12 - p.bat_centre[0].ld();
    p.difference[0].st(t13);
    let t14 = p.ball_centre[1].ld();
    let t15 = t14 - p.bat_centre[1].ld();
    p.difference[1].st(t15);
}

/// `new_way`'s x87 block as plain Rust on the x87 model (19 instructions), same order and
/// memory places; `length` the bits of `flt_76516C`.
fn new_way_x87(p: &PongState, r: &PongRules, length: u32) {
    let t0 = p.difference[0].ld();
    let t1 = t0 * t0;
    let t2 = p.difference[1].ld();
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    let t6 = ld(length) / t5;
    let t7 = p.difference[0].ld();
    let t8 = t7 * t6;
    p.difference[0].st(t8);
    let t9 = p.difference[1].ld();
    let t10 = t6 * t9;
    p.difference[1].st(t10);
    let t11 = p.difference[0].ld();
    let t12 = t11 * r.way_scale[0].ld();
    p.difference[0].st(t12);
    let t13 = p.difference[1].ld();
    let t14 = t13 * r.way_scale[1].ld();
    p.difference[1].st(t14);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn centres_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x3760]",
            "fmul dword ptr [{mk} + 0x4286]",
            "fadd dword ptr [{mk} + 0x3748]",
            "fstp dword ptr [{mk} + 0x379C]",
            "fld dword ptr [{mk} + 0x3764]",
            "fmul dword ptr [{mk} + 0x428A]",
            "fadd dword ptr [{mk} + 0x374C]",
            "fstp dword ptr [{mk} + 0x37A0]",
            "fld dword ptr [{mk} + 0x3790]",
            "fmul dword ptr [{mk} + 0x428E]",
            "fadd dword ptr [{mk} + 0x3778]",
            "fstp dword ptr [{mk} + 0x37A8]",
            "fld dword ptr [{mk} + 0x3794]",
            "fmul dword ptr [{mk} + 0x4292]",
            "fadd dword ptr [{mk} + 0x377C]",
            "fstp dword ptr [{mk} + 0x37AC]",
            "fld dword ptr [{mk} + 0x379C]",
            "fsub dword ptr [{mk} + 0x37A8]",
            "fstp dword ptr [{mk} + 0x37B4]",
            "fld dword ptr [{mk} + 0x37A0]",
            "fsub dword ptr [{mk} + 0x37AC]",
            "fstp dword ptr [{mk} + 0x37B8]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn new_way_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x37B4]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x37B8]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fld dword ptr [{mk} + 0x37B4]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x37B4]",
            "fld dword ptr [{mk} + 0x37B8]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{mk} + 0x37B8]",
            "fld dword ptr [{mk} + 0x37B4]",
            "fmul dword ptr [{mk} + 0x429A]",
            "fstp dword ptr [{mk} + 0x37B4]",
            "fld dword ptr [{mk} + 0x37B8]",
            "fmul dword ptr [{mk} + 0x429E]",
            "fstp dword ptr [{mk} + 0x37B8]",
            mk = in(reg) mk,
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
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::pong::{rules_at, state_at};

    #[test]
    fn centres_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x3718,
                hi: 0x42F2,
                reads: &[
                    (0x3748, F32),
                    (0x374C, F32),
                    (0x3760, F32),
                    (0x3764, F32),
                    (0x3778, F32),
                    (0x377C, F32),
                    (0x3790, F32),
                    (0x3794, F32),
                    (0x379C, F32),
                    (0x37A0, F32),
                    (0x37A8, F32),
                    (0x37AC, F32),
                    (0x4286, F32),
                    (0x428A, F32),
                    (0x428E, F32),
                    (0x4292, F32),
                ],
                writes: &[
                    (0x379C, F32),
                    (0x37A0, F32),
                    (0x37A8, F32),
                    (0x37AC, F32),
                    (0x37B4, F32),
                    (0x37B8, F32),
                ],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::centres_x87(b[0]),
            |b| super::centres_x87(state_at(b[0]), &rules_at(b[0])),
        );
    }

    #[test]
    fn new_way_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x22C,
                hi: 0x42F2,
                reads: &[
                    (0x22C, F32),
                    (0x37B4, F32),
                    (0x37B8, F32),
                    (0x429A, F32),
                    (0x429E, F32),
                ],
                writes: &[(0x37B4, F32), (0x37B8, F32)],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::new_way_x87(b[0]),
            |b| {
                let length = unsafe {
                    (crate::pointers::host_at(b[0] + 0x22C) as *const u32).read_unaligned()
                };
                super::new_way_x87(state_at(b[0]), &rules_at(b[0]), length);
            },
        );
    }
}
