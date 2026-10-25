//! `marks::squares_spawn` (0x4C0507): new squares this frame. `dword_89CA30` plus 1; with the
//! x87 rounding set to chop throughout (put back at the end): a rate (the sine of
//! `flt_76923E` + `g_marksShapeTilt` times `flt_89CA34`, plus -(`flt_89CA34` times
//! `flt_811BE7`) times its cosine times `flt_8BC246`) into `flt_89CA38`; unless below
//! `flt_8BC24A`: its whole part (held to 0x80) of `marks::pick_square`s, and one more when
//! the squares' random (`dword_8BC252`, times 0xC5 plus 0x5D) bits 8 .. 23 are at most its
//! fraction times `flt_8BC24E`. (After the compare the original frees st(0) and moves the
//! stack's top on, with the stack already empty: kept.)
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4BFFEC` (0x4C0052). The x87 run is plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables; the picks their Rust versions (under the same rounding).

use crate::marks::pick_square::pick_square;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::scene::vector3::STATUS_RVA;

const FRAMES_RVA: Slot = Slot::at(0x89CA30); // dword_89CA30
const RATE_RVA: Slot = Slot::at(0x89CA38); // flt_89CA38
const SHARE_K_RVA: Slot = Slot::at(0x8BC24E); // flt_8BC24E
const RANDOM_RVA: Slot = Slot::at(0x8BC252); // dword_8BC252
const MOST: i32 = 0x80;

/// The original's 0x4C053C .. 0x4C059D: the rate, compared (`fstsw` into `g_fpuStatus`).
unsafe fn rate() {
    unsafe {
        rate_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn squares_spawn() {
    unsafe {
        game_write(FRAMES_RVA, game_read::<u32>(FRAMES_RVA).wrapping_add(1));
        let old = x87::set_chop();
        rate();
        let status = game_read::<u32>(STATUS_RVA) & 0x700;
        game_write(STATUS_RVA, status);
        if status != 0x100 {
            let v: u32 = game_read(RATE_RVA);
            let n = x87::to_int(v).min(MOST);
            for _ in 0..n.max(0) {
                pick_square();
            }
            let share = x87::fraction_times_to_int(v, game_read(SHARE_K_RVA));
            let s = game_read::<u32>(RANDOM_RVA)
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            game_write(RANDOM_RVA, s);
            if ((s >> 8) & 0xFFFF) as i32 <= share {
                pick_square();
            }
        }
        x87::set_control(old);
    }
}

use crate::marks::plain::{ld, st, st16};
use crate::math::x87_float as bh;

/// `rate`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
unsafe fn rate_x87(rc: usize, mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x42FE);
        let t1 = t0 + ld(mk, 0xACC83);
        let t2 = t1.sin();
        let t3 = t2 * ld(mk, 0x137AF4);
        st(mk, 0x137AF8, t3);
        let t4 = ld(mk, 0x137AF4);
        let t5 = t4 * ld(mk, 0xACCA7);
        let t6 = -t5;
        let t7 = ld(mk, 0x42FE);
        let t8 = t7 + ld(mk, 0xACC83);
        let t9 = t8.cos();
        let t10 = t6 * t9;
        let t11 = t10 * ld(mk, 0x157306);
        let t12 = t11 + ld(mk, 0x137AF8);
        st(mk, 0x137AF8, t12);
        let t13 = ld(mk, 0x137AF8);
        st16(rc, 0x31C814, bh::compare(t13, ld(mk, 0x15730A)));
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn rate_x87(rc: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x42FE]",
            "fadd dword ptr [{mk} + 0xACC83]",
            ".byte 0xD9, 0xFE",
            "fmul dword ptr [{mk} + 0x137AF4]",
            "fstp dword ptr [{mk} + 0x137AF8]",
            "fld dword ptr [{mk} + 0x137AF4]",
            "fmul dword ptr [{mk} + 0xACCA7]",
            ".byte 0xD9, 0xE0",
            "fld dword ptr [{mk} + 0x42FE]",
            "fadd dword ptr [{mk} + 0xACC83]",
            ".byte 0xD9, 0xFF",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{mk} + 0x157306]",
            "fadd dword ptr [{mk} + 0x137AF8]",
            "fstp dword ptr [{mk} + 0x137AF8]",
            "fld dword ptr [{mk} + 0x137AF8]",
            "fcomp dword ptr [{mk} + 0x15730A]",
            ".byte 0xDD, 0xC0",
            ".byte 0xD9, 0xF7",
            "fstsw word ptr [{rc} + 0x31C814]",
            rc = in(reg) rc,
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
    fn rate_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x31C814,
                    hi: 0x31C816,
                    reads: &[],
                    writes: &[(0x31C814, Status)],
                },
                Op::Base {
                    lo: 0x42FE,
                    hi: 0x15730E,
                    reads: &[
                        (0x42FE, F32),
                        (0xACC83, F32),
                        (0xACCA7, F32),
                        (0x137AF4, F32),
                        (0x137AF8, F32),
                        (0x157306, F32),
                        (0x15730A, F32),
                    ],
                    writes: &[(0x137AF8, F32)],
                },
            ],
            sincos: true,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::rate_x87(b[0], b[1]) },
            |b| unsafe { super::rate_x87(b[0], b[1]) },
        );
    }
}
