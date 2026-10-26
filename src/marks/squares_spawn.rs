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

use crate::errors::Refused;
use crate::marks::pick_square::pick_square;
use crate::marks::tilt_shape::SHAPE_TILT;
use crate::marks::work_store::MARKS_WORK;
use crate::math::x87;

const MOST: i32 = 0x80;

/// The original's 0x4C053C .. 0x4C059D: the rate, compared (`fstsw` into `g_fpuStatus`).
fn rate() {
    rate_x87(
        crate::race::store::asm_base() as usize,
        &KEPT,
        &SQUARE_TIMING,
        &SQUARE_STATE,
    )
}

/// The Rust version (see the header).
pub fn squares_spawn() -> Result<(), Refused> {
    {
        SQUARE_TIMING
            .frames
            .set(SQUARE_TIMING.frames.get().wrapping_add(1));
        let old = x87::set_chop();
        rate();
        let status = MARKS_WORK.fpu_status.get() & 0x700;
        MARKS_WORK.fpu_status.set(status);
        if status != 0x100 {
            let v: u32 = SQUARE_TIMING.spawn_rate.bits();
            let n = x87::to_int(v).min(MOST);
            for _ in 0..n.max(0) {
                pick_square()?;
            }
            let share = x87::fraction_times_to_int(v, SQUARE_STATE.share_k.bits());
            let s = SQUARE_STATE
                .spawn_random
                .get()
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            SQUARE_STATE.spawn_random.set(s);
            if ((s >> 8) & 0xFFFF) as i32 <= share {
                pick_square()?;
            }
        }
        x87::set_control(old);
    }
    Ok(())
}

use crate::marks::plain::st16;
use crate::marks::squares_store::{SQUARE_STATE, SQUARE_TIMING, SquareState, SquareTiming};
use crate::marks::view_vectors::{KEPT, KeptView};
use crate::math::x87_float as bh;

/// `rate`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
fn rate_x87(rc: usize, kv: &KeptView, sti: &SquareTiming, ss: &SquareState) {
    let t0 = bh::ld(kv.first_angle.bits());
    let t1 = t0 + bh::ld(SHAPE_TILT.tilt.bits());
    let t2 = t1.sin();
    let t3 = t2 * bh::ld(sti.rate.bits());
    sti.spawn_rate.set_bits(bh::st(t3));
    let t4 = bh::ld(sti.rate.bits());
    let t5 = t4 * bh::ld(SHAPE_TILT.point[2].bits());
    let t6 = -t5;
    let t7 = bh::ld(kv.first_angle.bits());
    let t8 = t7 + bh::ld(SHAPE_TILT.tilt.bits());
    let t9 = t8.cos();
    let t10 = t6 * t9;
    let t11 = t10 * bh::ld(ss.rate_constants[0].bits());
    let t12 = t11 + bh::ld(sti.spawn_rate.bits());
    sti.spawn_rate.set_bits(bh::st(t12));
    let t13 = bh::ld(sti.spawn_rate.bits());
    st16(
        rc,
        0x31C814,
        bh::compare(t13, bh::ld(ss.rate_constants[1].bits())),
    );
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn rate_x87(rc: usize, mk: usize) {
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

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;

    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::with_copy;
    use crate::marks::asm_test::{Op, Spec, differential};
    use crate::marks::squares_store::{SquareState, SquareTiming};
    use crate::marks::tilt_shape::{SHAPE_TILT, ShapeTilt};
    use crate::marks::typed_sync::{overlay, with_overlays};
    use crate::marks::view_vectors::KeptView;

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
                    lo: 0x42F6,
                    hi: 0x15732C,
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
            |b| reference::rate_x87(b[0], b[1]),
            |b| {
                with_copy(b[1], KeptView::START, |kv| {
                    with_copy::<SquareTiming, _>(b[1], SquareTiming::START, |sti| {
                        with_copy::<SquareState, _>(b[1], SquareState::START, |ss| {
                            with_overlays(b[1], &[overlay(&SHAPE_TILT, ShapeTilt::START)], || {
                                super::rate_x87(b[0], kv, sti, ss)
                            })
                        })
                    })
                })
            },
        );
    }
}
