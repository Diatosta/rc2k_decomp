//! `effects::windscreen_drops_spawn`: new windscreen drops this frame, under x87 chop rounding
//! (restored at the end). A spawn rate worked out from the spawn angle, the shape tilt and the
//! drops' `rate` is kept; unless it is below a limit, its whole part (held to 0x80) of
//! `effects::pick_windscreen_drop`s are made, and one more when the effects' random's bits 8 .. 23
//! are at most its fraction times a constant.
//!
//! The x87 run is plain Rust on the x87 model (`math::x87_float`); the picks run under the same
//! rounding.

use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_drops::pick_windscreen_drop;
use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::x87;

const MOST: i32 = 0x80;

/// The rate, compared (its status word goes to the shared FPU status cell).
fn rate(drops: &WindscreenDrops, effects: &crate::effects::owner::Effects) {
    rate_x87(drops, &effects.frame.kept_view, &effects.setup.work);
}

pub fn windscreen_drops_spawn(
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        drops
            .state
            .frames
            .set(drops.state.frames.get().wrapping_add(1));
        if !crate::effects::spawn_gate::DROPS.due(setup.clock.now.get()) {
            return Ok(());
        }
        let old = x87::set_chop();
        rate(drops, effects);
        let status = setup.work.fpu_status.get() & 0x700;
        setup.work.fpu_status.set(status);
        if status != 0x100 {
            let v = drops.state.spawn_rate.get();
            let n = x87::to_int(v).min(MOST);
            for _ in 0..n.max(0) {
                pick_windscreen_drop(drops, effects)?;
            }
            let share = x87::fraction_times_to_int(v, &drops.hex.share_k);
            let s = setup.work.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
            setup.work.seed.set(s);
            if ((s >> 8) & 0xFFFF) as i32 <= share {
                pick_windscreen_drop(drops, effects)?;
            }
        }
        x87::set_control(old);
    }
    Ok(())
}

use crate::effects::plain::ld_cell;
use crate::effects::view_vectors::KeptView;
use crate::math::x87_float as bh;

/// `rate`'s arithmetic as plain Rust on the x87 model, in the same order.
fn rate_x87(drops: &WindscreenDrops, kv: &KeptView, work: &crate::effects::work_store::MarksWork) {
    let t0 = kv.first_angle.ld();
    let t1 = t0 + drops.tilt.tilt.ld();
    let t2 = t1.sin();
    let t3 = t2 * drops.state.rate.ld();
    drops.state.spawn_rate.st(t3);
    let t4 = drops.state.rate.ld();
    let t5 = t4 * drops.tilt.point[2].ld();
    let t6 = -t5;
    let t7 = kv.first_angle.ld();
    let t8 = t7 + drops.tilt.tilt.ld();
    let t9 = t8.cos();
    let t10 = t6 * t9;
    let t11 = t10 * ld_cell(&drops.hex.spawn_k[0]);
    let t12 = t11 + drops.state.spawn_rate.ld();
    drops.state.spawn_rate.st(t12);
    let t13 = drops.state.spawn_rate.ld();
    crate::effects::plain::set_status(work, bh::compare(t13, ld_cell(&drops.hex.spawn_k[1])));
}

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_layout::with_laid;
    use crate::effects::asm_test::Kind::*;

    use crate::effects::asm_test::{Op, Spec, recorded_writes};

    use crate::effects::typed_sync::{overlay_laid, with_overlays};
    use crate::effects::view_vectors::KeptView;

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
                    hi: 0xC9402,
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
        recorded_writes("effects.windscreen_drops_spawn.rate_x87", &SPEC, |b| {
            with_laid::<KeptView, _>(b[1], |kv| {
                with_overlays(
                    b[1],
                    &[
                        overlay_laid(&crate::test_support::own::effects().windscreen_drops.tilt),
                        overlay_laid(&crate::test_support::own::effects().windscreen_drops.state),
                        overlay_laid(&crate::test_support::own::effects().windscreen_drops.hex),
                    ],
                    || {
                        super::rate_x87(
                            &crate::test_support::own::effects().windscreen_drops,
                            kv,
                            &crate::test_support::own::effects().setup.work,
                        );
                    },
                );
            });
        });
    }
}
