//! `effects::windscreen_drops_spawn` (0x4BC5D9): new windscreen drops this frame. `dword_814724` plus 1;
//! with the x87 rounding set to chop throughout (put back at the end): a rate (the sine of
//! `flt_76923E` + `g_marksShapeTilt` times `flt_814728`, plus -(`flt_814728` times
//! `flt_811BE7`) times its cosine times `flt_82E2AA`) into `flt_81472C`; unless below
//! `flt_82E2AE`: its whole part (held to 0x80) of `effects::pick_windscreen_drop`s, and one more when
//! the marks' random's bits 8 .. 23 are at most its fraction times `flt_82E2B2`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BC0B1` (0x4BC225). The x87 run is plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables; the picks their Rust versions (under the same rounding).

use crate::effects::pick_windscreen_drop::pick_windscreen_drop;
use crate::effects::tilt_shape::SHAPE_TILT;
use crate::effects::windscreen_drop_state::WINDSCREEN_DROP_STATE;
use crate::effects::windscreen_drop_vars::HEX;
use crate::effects::work_store::MARKS_WORK;
use crate::errors::Refused;
use crate::math::x87;

const MOST: i32 = 0x80;

/// The original's 0x4BC60E .. 0x4BC66B: the rate, compared (`fstsw` into `g_fpuStatus`).
fn rate() {
    rate_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        &KEPT,
    );
}

/// The Rust version (see the header).
pub fn windscreen_drops_spawn() -> Result<(), Refused> {
    {
        WINDSCREEN_DROP_STATE
            .frames
            .set(WINDSCREEN_DROP_STATE.frames.get().wrapping_add(1));
        let old = x87::set_chop();
        rate();
        let status = MARKS_WORK.fpu_status.get() & 0x700;
        MARKS_WORK.fpu_status.set(status);
        if status != 0x100 {
            let v: u32 = WINDSCREEN_DROP_STATE.spawn_rate.bits();
            let n = x87::to_int(v).min(MOST);
            for _ in 0..n.max(0) {
                pick_windscreen_drop()?;
            }
            let share = x87::fraction_times_to_int(v, HEX.share_k.bits());
            let s = MARKS_WORK.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
            MARKS_WORK.seed.set(s);
            if ((s >> 8) & 0xFFFF) as i32 <= share {
                pick_windscreen_drop()?;
            }
        }
        x87::set_control(old);
    }
    Ok(())
}

use crate::effects::plain::ld;
use crate::effects::view_vectors::{KEPT, KeptView};
use crate::math::x87_float as bh;

/// `rate`'s x87 block as plain Rust on the x87 model (18 instructions), same order and memory places.
fn rate_x87(hx: usize, kv: &KeptView) {
    let t0 = bh::ld(kv.first_angle.bits());
    let t1 = t0 + bh::ld(SHAPE_TILT.tilt.bits());
    let t2 = t1.sin();
    let t3 = t2 * bh::ld(WINDSCREEN_DROP_STATE.rate.bits());
    WINDSCREEN_DROP_STATE.spawn_rate.set_bits(bh::st(t3));
    let t4 = bh::ld(WINDSCREEN_DROP_STATE.rate.bits());
    let t5 = t4 * bh::ld(SHAPE_TILT.point[2].bits());
    let t6 = -t5;
    let t7 = bh::ld(kv.first_angle.bits());
    let t8 = t7 + bh::ld(SHAPE_TILT.tilt.bits());
    let t9 = t8.cos();
    let t10 = t6 * t9;
    let t11 = t10 * ld(hx, 0xC936A);
    let t12 = t11 + bh::ld(WINDSCREEN_DROP_STATE.spawn_rate.bits());
    WINDSCREEN_DROP_STATE.spawn_rate.set_bits(bh::st(t12));
    let t13 = bh::ld(WINDSCREEN_DROP_STATE.spawn_rate.bits());
    crate::effects::plain::set_status(bh::compare(t13, ld(hx, 0xC936E)));
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
            "fmul dword ptr [{mk} + 0xAF7E8]",
            "fstp dword ptr [{mk} + 0xAF7EC]",
            "fld dword ptr [{mk} + 0xAF7E8]",
            "fmul dword ptr [{mk} + 0xACCA7]",
            ".byte 0xD9, 0xE0",
            "fld dword ptr [{mk} + 0x42FE]",
            "fadd dword ptr [{mk} + 0xACC83]",
            ".byte 0xD9, 0xFF",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{mk} + 0xC936A]",
            "fadd dword ptr [{mk} + 0xAF7EC]",
            "fstp dword ptr [{mk} + 0xAF7EC]",
            "fld dword ptr [{mk} + 0xAF7EC]",
            "fcomp dword ptr [{mk} + 0xC936E]",
            "fstsw word ptr [{rc} + 0x31C814]",
            rc = in(reg) rc,
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
    use crate::effects::asm_test::with_copy;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::tilt_shape::{SHAPE_TILT, ShapeTilt};
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::view_vectors::KeptView;
    use crate::effects::windscreen_drop_state::{WINDSCREEN_DROP_STATE, WindscreenDropState};

    #[test]
    fn rate_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[],
                    writes: &[(0x31C814, Status)],
                },
                Op::Base {
                    lo: 0x42F6,
                    hi: 0xC9372,
                    reads: &[
                        (0x42FE, F32),
                        (0xACC83, F32),
                        (0xACCA7, F32),
                        (0xAF7E8, F32),
                        (0xAF7EC, F32),
                        (0xC936A, F32),
                        (0xC936E, F32),
                    ],
                    writes: &[(0xAF7EC, F32)],
                },
            ],
            sincos: true,
        };
        differential(
            &SPEC,
            |b| reference::rate_x87(b[0], b[1]),
            |b| {
                with_copy(b[1], KeptView::START, |kv| {
                    with_overlays(
                        b[1],
                        &[
                            overlay(&SHAPE_TILT, ShapeTilt::START),
                            overlay(&WINDSCREEN_DROP_STATE, WindscreenDropState::START),
                        ],
                        || super::rate_x87(b[1], kv),
                    );
                });
            },
        );
    }
}
