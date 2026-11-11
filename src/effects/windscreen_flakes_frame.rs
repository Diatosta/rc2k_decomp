//! `effects::windscreen_flakes_frame` (0x4BF60F): the windscreen flakes' frame (as
//! `effects::draw_plane`'s, with the rate times one constant and the lengths over two more):
//! the rate, the place and two axes from the settings, made one long and square, scaled
//! (`effects::scale_axes`); nothing more unless the place's way faces the third axis away;
//! then `effects::run_pong`; unless the rate times a constant comes to 0 (rounded, as the x87
//! is set) `effects::run_windscreen_flakes`, and with the settings' +0x2744 and the drift
//! switch `effects::windscreen_flakes_step`.//!
//! Screen: the snow settling on the windscreen in the in-car view (confirmed: gone with `draw_windscreen_flake` cut). Drawn as squares.
//!
//! Works on the effects settings.
//!
//! Caller: `effects::draw_overlays` The x87 runs are plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables; the vector helpers,
//! `scale_axes` and `windscreen_flakes_step` their Rust versions. Refuses null settings and
//! `windscreen_flakes_step`'s refusals.

use crate::effects::cell_x87::CellX87;
use crate::effects::mask_store::Masks;
use crate::effects::plain::ldi_cell;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes::windscreen_flakes_step;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::effects_settings_store::EffectsSettings;
use crate::scene::cross_product3::cross;
use crate::scene::vector3::normalise;

/// The original's 0x4BF615 .. 0x4BF6AB: the rate (`WindscreenFlakeTiming.rate`), the place and the first axis from the settings, its length over `WindscreenFlakeState.length_divisors[0]` into `WindscreenFlakeTables.lengths[0]`.
fn frame_axes(fl: &WindscreenFlakes, m: &EffectsSettings) {
    frame_axes_x87(m, &fl.timing, &fl.tables, &fl.state);
}

/// The original's 0x4BF6B8 .. 0x4BF718: the second axis, its length over `WindscreenFlakeState.length_divisors[1]` into `WindscreenFlakeTables.lengths[1]`.
fn frame_up(fl: &WindscreenFlakes, m: &EffectsSettings) {
    frame_up_x87(m, &fl.tables, &fl.state);
}

/// The original's 0x4BF769 .. 0x4BF78D: the place as floats into `WindscreenDropVars.place[0]` ...
fn frame_place(drops: &WindscreenDrops, m: &EffectsSettings) {
    frame_place_x87(drops, m);
}

/// The original's 0x4BF7A4 .. 0x4BF7BE: the place's way dotted with the third axis into `WindscreenDropState.facing`.
fn frame_facing(drops: &WindscreenDrops, fl: &WindscreenFlakes) {
    frame_facing_x87(drops, &fl.tables);
}

/// The flakes are tinted by `ctx`'s view's light.
pub fn windscreen_flakes_frame(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    masks: &Masks,
    drops: &WindscreenDrops,
    pong: &crate::effects::pong::Pong,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (frame, scene) = (&ctx.race.frame, ctx.scene);
    let m = effects.settings()?;
    frame_axes(fl, m);
    let [a0, a1, a2] = [0, 1, 2].map(|n| &fl.tables.axes[n]);
    normalise(a0);
    frame_up(fl, m);
    normalise(a1);
    cross(a1, a0, a2);
    normalise(a2);
    cross(a0, a2, a1);
    crate::effects::scale_axes::scale_axes(fl);
    frame_place(drops, m);
    normalise(&drops.hex.place);
    frame_facing(drops, fl);
    if !drops.state.facing.get().is_sign_negative() {
        return Ok(());
    }
    crate::effects::run_pong::run_pong(frame, pong, fl, scene, effects)?;
    if x87::mul_to_int(&fl.timing.rate, &fl.state.share) == 0 {
        return Ok(());
    }
    crate::effects::run_windscreen_flakes::run_windscreen_flakes(
        masks,
        drops,
        fl,
        ctx.view.light.view_light.bits(),
        effects,
        scene,
        &ctx.race.shared,
    )?;
    let marks = effects.settings()?;
    if marks.windscreen_wet.get() == 0 {
        return Ok(());
    }
    if effects.setup.kind_floats.running.get() == 0 {
        return Ok(());
    }
    windscreen_flakes_step(masks, drops, fl)?;
    Ok(())
}

use crate::effects::plain::{ld_cell, st_cell};
use crate::effects::windscreen_flakes_store::{
    WindscreenFlakeState, WindscreenFlakeTables, WindscreenFlakeTiming,
};

/// `frame_axes`'s x87 block as plain Rust on the x87 model (29 instructions), same order and memory places.
fn frame_axes_x87(
    m: &EffectsSettings,
    sti: &WindscreenFlakeTiming,
    sq: &WindscreenFlakeTables,
    ss: &WindscreenFlakeState,
) {
    let t0 = ldi_cell(&m.windscreen_flakes_rate_factor);
    let t1 = t0 * ss.rate_share.ld();
    sti.rate.st(t1);
    let t2 = ldi_cell(&m.light_point_b_negated[0]);
    let t3 = ldi_cell(&m.light_point_b_negated[1]);
    let t4 = ldi_cell(&m.light_point_b_negated[2]);
    sq.place[2].st(t4);
    sq.place[1].st(t3);
    sq.place[0].st(t2);
    let t5 = ldi_cell(&m.light_point_b[0]);
    let t6 = t5 - ldi_cell(&m.light_point_b_negated[0]);
    sq.axes[0][0].st(t6);
    let t7 = ldi_cell(&m.light_point_b[1]);
    let t8 = t7 - ldi_cell(&m.light_point_b_negated[1]);
    sq.axes[0][1].st(t8);
    let t9 = ldi_cell(&m.light_point_b[2]);
    let t10 = t9 - ldi_cell(&m.light_point_b_negated[2]);
    sq.axes[0][2].st(t10);
    let t11 = sq.axes[0][0].ld();
    let t12 = t11 * t11;
    let t13 = sq.axes[0][1].ld();
    let t14 = t13 * t13;
    let t15 = sq.axes[0][2].ld();
    let t16 = t15 * t15;
    let t17 = t14 + t16;
    let t18 = t12 + t17;
    let t19 = t18.sqrt();
    let t20 = t19 / ss.length_divisors[0].ld();
    sq.lengths[0].st(t20);
}

/// `frame_up`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
fn frame_up_x87(m: &EffectsSettings, sq: &WindscreenFlakeTables, ss: &WindscreenFlakeState) {
    let t0 = ldi_cell(&m.light_point_a_negated[0]);
    let t1 = t0 - ldi_cell(&m.light_point_b_negated[0]);
    sq.axes[1][0].st(t1);
    let t2 = ldi_cell(&m.light_point_a_negated[1]);
    let t3 = t2 - ldi_cell(&m.light_point_b_negated[1]);
    sq.axes[1][1].st(t3);
    let t4 = ldi_cell(&m.light_point_a_negated[2]);
    let t5 = t4 - ldi_cell(&m.light_point_b_negated[2]);
    sq.axes[1][2].st(t5);
    let t6 = sq.axes[1][0].ld();
    let t7 = t6 * t6;
    let t8 = sq.axes[1][1].ld();
    let t9 = t8 * t8;
    let t10 = sq.axes[1][2].ld();
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 / ss.length_divisors[1].ld();
    sq.lengths[1].st(t15);
}

/// `frame_place`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
fn frame_place_x87(drops: &WindscreenDrops, m: &EffectsSettings) {
    let t0 = ldi_cell(&m.light_point_b_negated[0]);
    st_cell(&drops.hex.place[0], t0);
    let t1 = ldi_cell(&m.light_point_b_negated[1]);
    st_cell(&drops.hex.place[1], t1);
    let t2 = ldi_cell(&m.light_point_b_negated[2]);
    st_cell(&drops.hex.place[2], t2);
}

/// `frame_facing`'s x87 block as plain Rust on the x87 model (9 instructions), same order and memory places.
fn frame_facing_x87(drops: &WindscreenDrops, sq: &WindscreenFlakeTables) {
    let t0 = ld_cell(&drops.hex.place[0]);
    let t1 = t0 * sq.axes[2][0].ld();
    let t2 = ld_cell(&drops.hex.place[1]);
    let t3 = t2 * sq.axes[2][1].ld();
    let t4 = ld_cell(&drops.hex.place[2]);
    let t5 = t4 * sq.axes[2][2].ld();
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    drops.state.facing.st(t7);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn frame_axes_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{s} + 0x2740]",
            "fmul dword ptr [{mk} + 0x15725E]",
            "fstp dword ptr [{mk} + 0x137AF4]",
            "fild dword ptr [{s} + 0x2720]",
            "fild dword ptr [{s} + 0x2724]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0x137FBE]",
            "fstp dword ptr [{mk} + 0x137FBA]",
            "fstp dword ptr [{mk} + 0x137FB6]",
            "fild dword ptr [{s} + 0x272C]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0x137FDA]",
            "fild dword ptr [{s} + 0x2730]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0x137FDE]",
            "fild dword ptr [{s} + 0x2734]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0x137FE2]",
            "fld dword ptr [{mk} + 0x137FDA]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FDE]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FE2]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{mk} + 0x157262]",
            "fstp dword ptr [{mk} + 0x137FAE]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn frame_up_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{s} + 0x2708]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0x137FE6]",
            "fild dword ptr [{s} + 0x270C]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0x137FEA]",
            "fild dword ptr [{s} + 0x2710]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0x137FEE]",
            "fld dword ptr [{mk} + 0x137FE6]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FEA]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FEE]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{mk} + 0x157266]",
            "fstp dword ptr [{mk} + 0x137FB2]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn frame_place_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0xB1262]",
            "fild dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0xB1266]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0xB126A]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn frame_facing_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0xB1262]",
            "fmul dword ptr [{mk} + 0x137FF2]",
            "fld dword ptr [{mk} + 0xB1266]",
            "fmul dword ptr [{mk} + 0x137FF6]",
            "fld dword ptr [{mk} + 0xB126A]",
            "fmul dword ptr [{mk} + 0x137FFA]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0xAF7D8]",
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
    use crate::effects::asm_test::with_settings;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::windscreen_drop_state::WindscreenDropState;
    use crate::effects::windscreen_drop_vars::WindscreenDropVars;
    use crate::effects::windscreen_flakes_store::{
        WindscreenFlakeState, WindscreenFlakeTables, WindscreenFlakeTiming,
    };

    #[test]
    fn frame_axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137AF0,
                    hi: 0x157326,
                    reads: &[
                        (0x137FDA, F32),
                        (0x137FDE, F32),
                        (0x137FE2, F32),
                        (0x15725E, F32),
                        (0x157262, F32),
                    ],
                    writes: &[
                        (0x137AF4, F32),
                        (0x137FAE, F32),
                        (0x137FB6, F32),
                        (0x137FBA, F32),
                        (0x137FBE, F32),
                        (0x137FDA, F32),
                        (0x137FDE, F32),
                        (0x137FE2, F32),
                    ],
                },
                Op::Base {
                    lo: 0,
                    hi: 0x2768,
                    reads: &[
                        (0x2720, I32),
                        (0x2724, I32),
                        (0x2728, I32),
                        (0x272C, I32),
                        (0x2730, I32),
                        (0x2734, I32),
                        (0x2740, I32),
                    ],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::frame_axes_x87(b[0], b[1]),
            |b| {
                with_copy::<WindscreenFlakeTiming, _>(b[0], WindscreenFlakeTiming::START, |sti| {
                    with_copy::<WindscreenFlakeTables, _>(
                        b[0],
                        WindscreenFlakeTables::START,
                        |sq| {
                            with_copy::<WindscreenFlakeState, _>(
                                b[0],
                                WindscreenFlakeState::START,
                                |ss| with_settings(b[1], |m| super::frame_axes_x87(m, sti, sq, ss)),
                            );
                        },
                    );
                });
            },
        );
    }

    #[test]
    fn frame_up_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137B8A,
                    hi: 0x157326,
                    reads: &[
                        (0x137FE6, F32),
                        (0x137FEA, F32),
                        (0x137FEE, F32),
                        (0x157266, F32),
                    ],
                    writes: &[
                        (0x137FB2, F32),
                        (0x137FE6, F32),
                        (0x137FEA, F32),
                        (0x137FEE, F32),
                    ],
                },
                Op::Base {
                    lo: 0,
                    hi: 0x2768,
                    reads: &[
                        (0x2708, I32),
                        (0x270C, I32),
                        (0x2710, I32),
                        (0x2720, I32),
                        (0x2724, I32),
                        (0x2728, I32),
                    ],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::frame_up_x87(b[0], b[1]),
            |b| {
                with_copy::<WindscreenFlakeTables, _>(b[0], WindscreenFlakeTables::START, |sq| {
                    with_copy::<WindscreenFlakeState, _>(b[0], WindscreenFlakeState::START, |ss| {
                        with_settings(b[1], |m| super::frame_up_x87(m, sq, ss));
                    });
                });
            },
        );
    }

    #[test]
    fn frame_place_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xB1252,
                    hi: 0xC9402,
                    reads: &[],
                    writes: &[(0xB1262, F32), (0xB1266, F32), (0xB126A, F32)],
                },
                Op::Base {
                    lo: 0,
                    hi: 0x2768,
                    reads: &[(0x2720, I32), (0x2724, I32), (0x2728, I32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::frame_place_x87(b[0], b[1]),
            |b| {
                with_overlays(
                    b[0],
                    &[overlay(
                        &crate::effects::owner::effects().windscreen_drops.hex,
                        WindscreenDropVars::START,
                    )],
                    || {
                        with_settings(b[1], |m| {
                            super::frame_place_x87(
                                &crate::effects::owner::effects().windscreen_drops,
                                m,
                            );
                        });
                    },
                );
            },
        );
    }

    #[test]
    fn frame_facing_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xAF7D0,
                hi: 0x13822A,
                reads: &[
                    (0xB1262, F32),
                    (0xB1266, F32),
                    (0xB126A, F32),
                    (0x137FF2, F32),
                    (0x137FF6, F32),
                    (0x137FFA, F32),
                ],
                writes: &[(0xAF7D8, F32)],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::frame_facing_x87(b[0]),
            |b| {
                with_overlays(
                    b[0],
                    &[
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.state,
                            WindscreenDropState::START,
                        ),
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.hex,
                            WindscreenDropVars::START,
                        ),
                    ],
                    || {
                        with_copy::<WindscreenFlakeTables, _>(
                            b[0],
                            WindscreenFlakeTables::START,
                            |m| {
                                super::frame_facing_x87(
                                    &crate::effects::owner::effects().windscreen_drops,
                                    m,
                                );
                            },
                        );
                    },
                );
            },
        );
    }
}
