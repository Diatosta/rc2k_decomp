//! `effects::windscreen_flakes_spawn` (0x4C0507): new windscreen flakes this frame. The frame
//! counter plus 1; with the x87 rounding set to chop throughout (put back at the end): a rate
//! (the sine of the spawn angle + the shape tilt times the flakes' `rate`, plus -(`rate` times
//! a constant) times its cosine times a factor) is kept; unless below a limit: its whole part
//! (held to 0x80) of `effects::pick_windscreen_flake`s, and one more when the windscreen
//! flakes' random (times 0xC5 plus 0x5D) bits 8 .. 23 are at most its fraction times a
//! constant. (After the compare the original frees st(0) and moves the stack's top on, with
//! the stack already empty: kept.)
//!
//! Caller: `effects::run_windscreen_flakes` The x87 run is plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables; the picks their Rust
//! versions (under the same rounding).

use crate::effects::cell_x87::CellX87;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes::pick_windscreen_flake;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner_shared::RaceShared;

const MOST: i32 = 0x80;

/// The rate, compared (its status word goes to the shared FPU status cell).
fn rate(drops: &WindscreenDrops, fl: &WindscreenFlakes, effects: &crate::effects::owner::Effects) {
    rate_x87(drops, &effects.frame.kept_view, &fl.timing, &fl.state);
}

pub fn windscreen_flakes_spawn(
    shared: &RaceShared,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        fl.timing.frames.set(fl.timing.frames.get().wrapping_add(1));
        let old = x87::set_chop();
        rate(drops, fl, effects);
        let status = setup.work.fpu_status.get() & 0x700;
        setup.work.fpu_status.set(status);
        if status != 0x100 {
            let v: u32 = fl.timing.spawn_rate.bits();
            let n = x87::to_int(v).min(MOST);
            for _ in 0..n.max(0) {
                pick_windscreen_flake(shared, fl)?;
            }
            let share = x87::fraction_times_to_int(v, &fl.state.share_k);
            let s = fl
                .state
                .spawn_random
                .get()
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            fl.state.spawn_random.set(s);
            if ((s >> 8) & 0xFFFF) as i32 <= share {
                pick_windscreen_flake(shared, fl)?;
            }
        }
        x87::set_control(old);
    }
    Ok(())
}

use crate::effects::view_vectors::KeptView;
use crate::effects::windscreen_flakes_store::{WindscreenFlakeState, WindscreenFlakeTiming};
use crate::math::x87_float as bh;

/// `rate`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
fn rate_x87(
    drops: &WindscreenDrops,
    kv: &KeptView,
    sti: &WindscreenFlakeTiming,
    ss: &WindscreenFlakeState,
) {
    let t0 = kv.first_angle.ld();
    let t1 = t0 + drops.tilt.tilt.ld();
    let t2 = t1.sin();
    let t3 = t2 * sti.rate.ld();
    sti.spawn_rate.st(t3);
    let t4 = sti.rate.ld();
    let t5 = t4 * drops.tilt.point[2].ld();
    let t6 = -t5;
    let t7 = kv.first_angle.ld();
    let t8 = t7 + drops.tilt.tilt.ld();
    let t9 = t8.cos();
    let t10 = t6 * t9;
    let t11 = t10 * ss.rate_constants[0].ld();
    let t12 = t11 + sti.spawn_rate.ld();
    sti.spawn_rate.st(t12);
    let t13 = sti.spawn_rate.ld();
    crate::effects::plain::set_status(bh::compare(t13, ss.rate_constants[1].ld()));
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
    use crate::effects::tilt_shape::ShapeTilt;
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::view_vectors::KeptView;
    use crate::effects::windscreen_flakes_store::{WindscreenFlakeState, WindscreenFlakeTiming};

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
                    with_copy::<WindscreenFlakeTiming, _>(
                        b[1],
                        WindscreenFlakeTiming::START,
                        |sti| {
                            with_copy::<WindscreenFlakeState, _>(
                                b[1],
                                WindscreenFlakeState::START,
                                |ss| {
                                    with_overlays(
                                        b[1],
                                        &[overlay(
                                            &crate::effects::owner::effects().windscreen_drops.tilt,
                                            ShapeTilt::START,
                                        )],
                                        || {
                                            super::rate_x87(
                                                &crate::effects::owner::effects().windscreen_drops,
                                                kv,
                                                sti,
                                                ss,
                                            );
                                        },
                                    );
                                },
                            );
                        },
                    );
                });
            },
        );
    }
}
