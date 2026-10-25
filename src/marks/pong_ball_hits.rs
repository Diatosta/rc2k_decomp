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

use crate::marks::pong;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

/// The original's 0x4B8105 .. 0x4B8189: the centres and their difference.
unsafe fn centres() {
    unsafe { centres_x87(crate::marks::store::base() as usize) }
}

/// The original's 0x4B81A4 .. 0x4B81FE: the difference made one long, scaled.
unsafe fn new_way() {
    unsafe { new_way_x87(crate::marks::store::base() as usize) }
}

const ACROSS_RVA: Slot = Slot::at(0x7686F4); // flt_7686F4
const ALONG_RVA: Slot = Slot::at(0x7686F8); // flt_7686F8
const NUDGE_RVA: Slot = Slot::at(0x7691D6); // flt_7691D6

/// The Rust version (see the header).
pub unsafe fn pong_ball_hits() {
    let (over, _) = pong::overlap();
    if !over {
        return;
    }
    unsafe {
        game_write(
            pong::HITS_RVA,
            game_read::<u32>(pong::HITS_RVA).wrapping_add(1),
        );
        game_write(
            pong::STEPS_RVA,
            game_read::<u32>(pong::STEPS_RVA).wrapping_add(4),
        );
        for n in 0..2 {
            let v = x87::sub(game_read(pong::BALL[n]), game_read(pong::BALL_WAY[n]));
            game_write(pong::BALL[n], v);
        }
        centres();
        if game_read::<u32>(ACROSS_RVA) == 0 {
            game_write(
                ACROSS_RVA,
                x87::sub(game_read(ACROSS_RVA), game_read(NUDGE_RVA)),
            );
        }
        new_way();
        game_write(pong::BALL_WAY[0], game_read::<u32>(ACROSS_RVA));
        game_write(pong::BALL_WAY[1], game_read::<u32>(ALONG_RVA));
    }
}

use crate::marks::plain::{ld, st};

/// `centres`'s x87 block as plain Rust on the x87 model (22 instructions), same order and memory places.
unsafe fn centres_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x3760);
        let t1 = t0 * ld(mk, 0x4286);
        let t2 = t1 + ld(mk, 0x3748);
        st(mk, 0x379C, t2);
        let t3 = ld(mk, 0x3764);
        let t4 = t3 * ld(mk, 0x428A);
        let t5 = t4 + ld(mk, 0x374C);
        st(mk, 0x37A0, t5);
        let t6 = ld(mk, 0x3790);
        let t7 = t6 * ld(mk, 0x428E);
        let t8 = t7 + ld(mk, 0x3778);
        st(mk, 0x37A8, t8);
        let t9 = ld(mk, 0x3794);
        let t10 = t9 * ld(mk, 0x4292);
        let t11 = t10 + ld(mk, 0x377C);
        st(mk, 0x37AC, t11);
        let t12 = ld(mk, 0x379C);
        let t13 = t12 - ld(mk, 0x37A8);
        st(mk, 0x37B4, t13);
        let t14 = ld(mk, 0x37A0);
        let t15 = t14 - ld(mk, 0x37AC);
        st(mk, 0x37B8, t15);
    }
}

/// `new_way`'s x87 block as plain Rust on the x87 model (19 instructions), same order and memory places.
unsafe fn new_way_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x37B4);
        let t1 = t0 * t0;
        let t2 = ld(mk, 0x37B8);
        let t3 = t2 * t2;
        let t4 = t1 + t3;
        let t5 = t4.sqrt();
        let t6 = ld(mk, 0x22C) / t5;
        let t7 = ld(mk, 0x37B4);
        let t8 = t7 * t6;
        st(mk, 0x37B4, t8);
        let t9 = ld(mk, 0x37B8);
        let t10 = t6 * t9;
        st(mk, 0x37B8, t10);
        let t11 = ld(mk, 0x37B4);
        let t12 = t11 * ld(mk, 0x429A);
        st(mk, 0x37B4, t12);
        let t13 = ld(mk, 0x37B8);
        let t14 = t13 * ld(mk, 0x429E);
        st(mk, 0x37B8, t14);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn centres_x87(mk: usize) {
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

    pub(super) unsafe fn new_way_x87(mk: usize) {
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

    #[test]
    fn centres_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x3748,
                hi: 0x4296,
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
            |b| unsafe { reference::centres_x87(b[0]) },
            |b| unsafe { super::centres_x87(b[0]) },
        );
    }

    #[test]
    fn new_way_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x22C,
                hi: 0x42A2,
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
            |b| unsafe { reference::new_way_x87(b[0]) },
            |b| unsafe { super::new_way_x87(b[0]) },
        );
    }
}
