//! `marks::pong_ball_hits` (0x4B8040): when the pong ball overlaps the bat (`marks::pong`'s
//! four compares): a hit (`dword_7687D0` plus 1), four more steps a frame
//! (`g_pongSteps`), the ball a step back, and its new way: from the bat's centre to
//! the ball's (`flt_7686DC` .. `flt_7686F8`; an across of exactly 0 bits less
//! `flt_7691D6`), made one long times `flt_76516C`, times `flt_7691DA` / `flt_7691DE`.
//!
//! In:  nothing
//! Out: nothing (the Rust callers read no register)
//!
//! Caller: `marks::move_pong_ball`. The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::marks::pong::{self, PONG, PONG_RULES, PongRules, PongState};
use crate::marks::tuning::MARKS_FLOATS;
use crate::math::x87;

/// The original's 0x4B8105 .. 0x4B8189: the centres and their difference.
fn centres() {
    centres_x87(&PONG, &PONG_RULES)
}

/// The original's 0x4B81A4 .. 0x4B81FE: the difference made one long, scaled.
fn new_way() {
    new_way_x87(&PONG, &PONG_RULES, MARKS_FLOATS.one.bits())
}

/// The Rust version (see the header).
pub fn pong_ball_hits() {
    let (over, _) = pong::overlap();
    if !over {
        return;
    }
    let p = &PONG;
    p.hits.set(p.hits.get().wrapping_add(1));
    p.steps.set(p.steps.get().wrapping_add(4));
    for n in 0..2 {
        p.ball[n].set_bits(x87::sub(p.ball[n].bits(), p.ball_way[n].bits()));
    }
    centres();
    if p.difference[0].bits() == 0 {
        p.difference[0].set_bits(x87::sub(p.difference[0].bits(), PONG_RULES.nudge.bits()));
    }
    new_way();
    p.ball_way[0].set_bits(p.difference[0].bits());
    p.ball_way[1].set_bits(p.difference[1].bits());
}

use crate::math::x87_float::{ld, st};

/// `centres`'s x87 block as plain Rust on the x87 model (22 instructions), same order and memory places.
fn centres_x87(p: &PongState, r: &PongRules) {
    let t0 = ld(p.ball_size[0].bits());
    let t1 = t0 * ld(r.centre[0].bits());
    let t2 = t1 + ld(p.ball[0].bits());
    p.ball_centre[0].set_bits(st(t2));
    let t3 = ld(p.ball_size[1].bits());
    let t4 = t3 * ld(r.centre[1].bits());
    let t5 = t4 + ld(p.ball[1].bits());
    p.ball_centre[1].set_bits(st(t5));
    let t6 = ld(p.bat_size[0].bits());
    let t7 = t6 * ld(r.centre[2].bits());
    let t8 = t7 + ld(p.bat[0].bits());
    p.bat_centre[0].set_bits(st(t8));
    let t9 = ld(p.bat_size[1].bits());
    let t10 = t9 * ld(r.centre[3].bits());
    let t11 = t10 + ld(p.bat[1].bits());
    p.bat_centre[1].set_bits(st(t11));
    let t12 = ld(p.ball_centre[0].bits());
    let t13 = t12 - ld(p.bat_centre[0].bits());
    p.difference[0].set_bits(st(t13));
    let t14 = ld(p.ball_centre[1].bits());
    let t15 = t14 - ld(p.bat_centre[1].bits());
    p.difference[1].set_bits(st(t15));
}

/// `new_way`'s x87 block as plain Rust on the x87 model (19 instructions), same order and
/// memory places; `length` the bits of `flt_76516C`.
fn new_way_x87(p: &PongState, r: &PongRules, length: u32) {
    let t0 = ld(p.difference[0].bits());
    let t1 = t0 * t0;
    let t2 = ld(p.difference[1].bits());
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    let t6 = ld(length) / t5;
    let t7 = ld(p.difference[0].bits());
    let t8 = t7 * t6;
    p.difference[0].set_bits(st(t8));
    let t9 = ld(p.difference[1].bits());
    let t10 = t6 * t9;
    p.difference[1].set_bits(st(t10));
    let t11 = ld(p.difference[0].bits());
    let t12 = t11 * ld(r.way_scale[0].bits());
    p.difference[0].set_bits(st(t12));
    let t13 = ld(p.difference[1].bits());
    let t14 = t13 * ld(r.way_scale[1].bits());
    p.difference[1].set_bits(st(t14));
}

#[cfg(test)]
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
            )
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
            )
        }
    }
}

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};
    use crate::marks::pong::{rules_at, state_at};

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
                super::new_way_x87(state_at(b[0]), &rules_at(b[0]), length)
            },
        );
    }
}
