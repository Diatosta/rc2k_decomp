//! `effects::windscreen_drops_top` (0x4BAC87): the windscreen drops' frame from the settings:
//! the vertex ring from its start (`buffer`); the ease and the rate; the place (the settings'
//! +0x2720 ..) and two axes (+0x272C .. and +0x2708 .. less it), their lengths' shares
//! (`shares`), made one long and square (`scene::normalise_vector3`,
//! `scene::cross_product3`); nothing more unless the place's way faces the third axis away
//! (a negative `facing` dot); the 0x30 windscreen drop points scaled into `scaled` and turned
//! onto the axes; unless the rate is 0: `effects::windscreen_drops_frame`, the vertices left
//! in the ring drawn (their count by `per_vertex`, `d3d::draw_marks_windscreen_drops`, the
//! platform's), and with the settings' +0x2744 and the drift on
//! `effects::windscreen_drops_step`; then `scene::set_texture` (the marks' texture) and the
//! polygon's vertex count 4.
//!
//! Screen: the rain drops on the windscreen in the in-car view (confirmed: gone with `d3d::draw_marks_windscreen_drops` cut). Drawn as hexagons.
//!
//! Works on the effects settings.
//!
//! Caller: `effects::draw_overlays` The x87 runs are plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables; the vector helpers
//! their Rust versions. Refuses null settings, before anything.

use crate::effects::cell_x87::CellX87;
use crate::effects::mask_store::Masks;
use crate::effects::plain::{ld_cell, ldi_cell, st_cell};
use crate::effects::setup::MarksClock;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::owner_frame::Frame;
use crate::scene::cross_product3::cross;
use crate::scene::vector3::normalise;
use crate::session::PlainFloat;

const POINTS: u32 = 0x30;

/// The ease (by the ticks), the rate, the place and the first axis from the settings, and
/// the first axis's length over its divisor into the first share.
fn frame_axes(
    drops: &WindscreenDrops,
    m: &EffectsSettings,
    effects: &crate::effects::owner::Effects,
) {
    frame_axes_x87(drops, m, &effects.setup.clock);
}

/// The second axis from the settings, its length over its divisor into the second share.
fn frame_up(drops: &WindscreenDrops, m: &EffectsSettings) {
    frame_up_x87(drops, m);
}

/// The place again as floats into the drops' `place`.
fn frame_place(drops: &WindscreenDrops, m: &EffectsSettings) {
    frame_place_x87(drops, m);
}

/// The place's way dotted with the third axis into `facing`.
fn frame_facing(drops: &WindscreenDrops) {
    frame_facing_x87(drops);
}

/// A point (x, y at p) turned onto the axes (x, y, z back at p).
fn point_turned(drops: &WindscreenDrops, p: &[PlainFloat; 3]) {
    point_turned_x87(drops, p);
}

pub fn windscreen_drops_top(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    frame: &Frame,
    masks: &Masks,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let m = effects.settings()?;
    drops.hex.buffer.set(0);
    frame_axes(drops, m, effects);
    let w = &drops.work.axes;
    let (a0, a1, a2) = (&w[0..3], &w[3..6], &w[6..9]);
    normalise(a0);
    frame_up(drops, m);
    normalise(a1);
    cross(a1, a0, a2);
    normalise(a2);
    cross(a0, a2, a1);
    frame_place(drops, m);
    normalise(&drops.hex.place);
    frame_facing(drops);
    if !drops.state.facing.get().is_sign_negative() {
        return Ok(());
    }
    let [kx, ky] = drops
        .work
        .shares
        .each_ref()
        .map(super::super::session::PlainFloat::bits);
    for n in 0..POINTS as usize {
        let (p, o) = (3 * n, 3 * n);
        drops.work.scaled[o].set(x87::mul(&drops.work.points[p], kx));
        drops.work.scaled[o + 1].set(x87::mul(&drops.work.points[p + 1], ky));
        drops.work.scaled[o + 2].set_bits(0);
    }
    for p in drops.work.scaled.as_chunks::<3>().0 {
        point_turned(drops, p);
    }
    if drops.state.rate.bits() != 0 {
        crate::effects::windscreen_drops_frame::windscreen_drops_frame(
            buffer, frame, masks, drops, scene, gfx, effects,
        )?;
        let at_now: u32 = drops.hex.buffer.get();
        if at_now != 0 {
            let quads =
                x87::int_over_int_to_int(at_now as i32, drops.hex.per_vertex.get() as i32) as u32;
            drops.hex.buffer.set(0);
            gfx.card
                .windscreen_drop_batch
                .index_count
                .set(quads.wrapping_mul(12));
            gfx.card
                .windscreen_drop_batch
                .vertex_count
                .set(quads.wrapping_mul(6));
            crate::effects::windscreen_drops_frame::draw_batch(gfx)?;
        }
        let marks = effects.settings()?;
        if marks.windscreen_wet.get() != 0 && effects.setup.kind_floats.running.get() != 0 {
            crate::effects::windscreen_drops::windscreen_drops_step(masks, drops)?;
        }
    }
    crate::scene::set_texture::set_texture(scene, scene.textures.kinds.kind_first_slots[202].get());
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set_bits(4u32);
    Ok(())
}

/// `frame_axes`'s x87 block as plain Rust on the x87 model (32 instructions), same order and memory places.
fn frame_axes_x87(drops: &WindscreenDrops, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&drops.hex.top_k[0]);
    let t1 = t0 * crate::math::x87_float::int(c.frame_ticks.get() as i32);
    st_cell(&drops.hex.ease, t1);
    let t2 = ldi_cell(&m.windscreen_drops_rate_factor);
    let t3 = t2 * ld_cell(&drops.hex.top_k[1]);
    drops.state.rate.st(t3);
    let t4 = ldi_cell(&m.light_point_b_negated[0]);
    let t5 = ldi_cell(&m.light_point_b_negated[1]);
    let t6 = ldi_cell(&m.light_point_b_negated[2]);
    drops.work.place[2].st(t6);
    drops.work.place[1].st(t5);
    drops.work.place[0].st(t4);
    let t7 = ldi_cell(&m.light_point_b[0]);
    let t8 = t7 - ldi_cell(&m.light_point_b_negated[0]);
    drops.work.axes[0].st(t8);
    let t9 = ldi_cell(&m.light_point_b[1]);
    let t10 = t9 - ldi_cell(&m.light_point_b_negated[1]);
    drops.work.axes[1].st(t10);
    let t11 = ldi_cell(&m.light_point_b[2]);
    let t12 = t11 - ldi_cell(&m.light_point_b_negated[2]);
    drops.work.axes[2].st(t12);
    let t13 = drops.work.axes[0].ld();
    let t14 = t13 * t13;
    let t15 = drops.work.axes[1].ld();
    let t16 = t15 * t15;
    let t17 = drops.work.axes[2].ld();
    let t18 = t17 * t17;
    let t19 = t16 + t18;
    let t20 = t14 + t19;
    let t21 = t20.sqrt();
    let t22 = t21 / ld_cell(&drops.hex.top_k[2]);
    drops.work.shares[0].st(t22);
}

/// `frame_up`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
fn frame_up_x87(drops: &WindscreenDrops, m: &EffectsSettings) {
    let t0 = ldi_cell(&m.light_point_a_negated[0]);
    let t1 = t0 - ldi_cell(&m.light_point_b_negated[0]);
    drops.work.axes[3].st(t1);
    let t2 = ldi_cell(&m.light_point_a_negated[1]);
    let t3 = t2 - ldi_cell(&m.light_point_b_negated[1]);
    drops.work.axes[4].st(t3);
    let t4 = ldi_cell(&m.light_point_a_negated[2]);
    let t5 = t4 - ldi_cell(&m.light_point_b_negated[2]);
    drops.work.axes[5].st(t5);
    let t6 = drops.work.axes[3].ld();
    let t7 = t6 * t6;
    let t8 = drops.work.axes[4].ld();
    let t9 = t8 * t8;
    let t10 = drops.work.axes[5].ld();
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 / ld_cell(&drops.hex.top_k[3]);
    drops.work.shares[1].st(t15);
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
fn frame_facing_x87(drops: &WindscreenDrops) {
    let t0 = ld_cell(&drops.hex.place[0]);
    let t1 = t0 * drops.work.axes[6].ld();
    let t2 = ld_cell(&drops.hex.place[1]);
    let t3 = t2 * drops.work.axes[7].ld();
    let t4 = ld_cell(&drops.hex.place[2]);
    let t5 = t4 * drops.work.axes[8].ld();
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    drops.state.facing.st(t7);
}

/// `point_turned`'s x87 block as plain Rust on the x87 model (19 instructions), same order and memory places.
fn point_turned_x87(drops: &WindscreenDrops, p: &[PlainFloat; 3]) {
    let t0 = ld_cell(&p[0]);
    let t1 = ld_cell(&p[1]);
    let t2 = t0 * drops.work.axes[0].ld();
    let t3 = t1 * drops.work.axes[3].ld();
    let t4 = t2 + t3;
    st_cell(&p[0], t4);
    let t5 = t0 * drops.work.axes[1].ld();
    let t6 = t1 * drops.work.axes[4].ld();
    let t7 = t5 + t6;
    st_cell(&p[1], t7);
    let t8 = t1 * drops.work.axes[5].ld();
    let t9 = t0 * drops.work.axes[2].ld();
    let t10 = t8 + t9;
    st_cell(&p[2], t10);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn frame_axes_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0xC932A]",
            "fimul dword ptr [{mk} + 0x90]",
            "fstp dword ptr [{mk} + 0xB12EA]",
            "fild dword ptr [{s} + 0x2738]",
            "fmul dword ptr [{mk} + 0xC932E]",
            "fstp dword ptr [{mk} + 0xAF7E8]",
            "fild dword ptr [{s} + 0x2720]",
            "fild dword ptr [{s} + 0x2724]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0xAFCD6]",
            "fstp dword ptr [{mk} + 0xAFCD2]",
            "fstp dword ptr [{mk} + 0xAFCCE]",
            "fild dword ptr [{s} + 0x272C]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0xAFCDA]",
            "fild dword ptr [{s} + 0x2730]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0xAFCDE]",
            "fild dword ptr [{s} + 0x2734]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0xAFCE2]",
            "fld dword ptr [{mk} + 0xAFCDA]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0xAFCDE]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0xAFCE2]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{mk} + 0xC9332]",
            "fstp dword ptr [{mk} + 0xAFCC6]",
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
            "fstp dword ptr [{mk} + 0xAFCE6]",
            "fild dword ptr [{s} + 0x270C]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0xAFCEA]",
            "fild dword ptr [{s} + 0x2710]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0xAFCEE]",
            "fld dword ptr [{mk} + 0xAFCE6]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0xAFCEA]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0xAFCEE]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{mk} + 0xC9336]",
            "fstp dword ptr [{mk} + 0xAFCCA]",
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
            "fmul dword ptr [{mk} + 0xAFCF2]",
            "fld dword ptr [{mk} + 0xB1266]",
            "fmul dword ptr [{mk} + 0xAFCF6]",
            "fld dword ptr [{mk} + 0xB126A]",
            "fmul dword ptr [{mk} + 0xAFCFA]",
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

    pub(super) fn point_turned_x87(mk: usize, p: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{p} + 0x0]",
            "fld dword ptr [{p} + 0x4]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0xAFCDA]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0xAFCE6]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{p} + 0x0]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0xAFCDE]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0xAFCEA]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{p} + 0x4]",
            "fmul dword ptr [{mk} + 0xAFCEE]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0xAFCE2]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{p} + 0x8]",
            mk = in(reg) mk,
            p = in(reg) p,
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
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::windscreen_drop_state::{WindscreenDropState, WindscreenDropWork};
    use crate::effects::windscreen_drop_vars::WindscreenDropVars;
    use crate::session::PlainFloat;

    #[test]
    fn frame_axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x90,
                    hi: 0xC9402,
                    reads: &[
                        (0x90, I32),
                        (0xAFCDA, F32),
                        (0xAFCDE, F32),
                        (0xAFCE2, F32),
                        (0xC932A, F32),
                        (0xC932E, F32),
                        (0xC9332, F32),
                    ],
                    writes: &[
                        (0xAF7E8, F32),
                        (0xAFCC6, F32),
                        (0xAFCCE, F32),
                        (0xAFCD2, F32),
                        (0xAFCD6, F32),
                        (0xAFCDA, F32),
                        (0xAFCDE, F32),
                        (0xAFCE2, F32),
                        (0xB12EA, F32),
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
                        (0x2738, I32),
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
                with_overlays(
                    b[0],
                    &[
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.state,
                            WindscreenDropState::START,
                        ),
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.work,
                            WindscreenDropWork::START,
                        ),
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.hex,
                            WindscreenDropVars::START,
                        ),
                    ],
                    || {
                        with_settings(b[1], |m| {
                            super::frame_axes_x87(
                                &crate::effects::owner::effects().windscreen_drops,
                                m,
                                crate::effects::setup::clock_at(b[0]),
                            );
                        });
                    },
                );
            },
        );
    }

    #[test]
    fn frame_up_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xAF832,
                    hi: 0xC9402,
                    reads: &[
                        (0xAFCE6, F32),
                        (0xAFCEA, F32),
                        (0xAFCEE, F32),
                        (0xC9336, F32),
                    ],
                    writes: &[
                        (0xAFCCA, F32),
                        (0xAFCE6, F32),
                        (0xAFCEA, F32),
                        (0xAFCEE, F32),
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
                with_overlays(
                    b[0],
                    &[
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.work,
                            WindscreenDropWork::START,
                        ),
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.hex,
                            WindscreenDropVars::START,
                        ),
                    ],
                    || {
                        with_settings(b[1], |m| {
                            super::frame_up_x87(
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
                hi: 0xC9402,
                reads: &[
                    (0xAFCF2, F32),
                    (0xAFCF6, F32),
                    (0xAFCFA, F32),
                    (0xB1262, F32),
                    (0xB1266, F32),
                    (0xB126A, F32),
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
                            &crate::effects::owner::effects().windscreen_drops.work,
                            WindscreenDropWork::START,
                        ),
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.hex,
                            WindscreenDropVars::START,
                        ),
                    ],
                    || super::frame_facing_x87(&crate::effects::owner::effects().windscreen_drops),
                );
            },
        );
    }

    #[test]
    fn point_turned_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xAF832,
                    hi: 0xB0250,
                    reads: &[
                        (0xAFCDA, F32),
                        (0xAFCDE, F32),
                        (0xAFCE2, F32),
                        (0xAFCE6, F32),
                        (0xAFCEA, F32),
                        (0xAFCEE, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xC,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32), (0x8, F32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::point_turned_x87(b[0], b[1]),
            |b| {
                with_overlays(
                    b[0],
                    &[overlay(
                        &crate::effects::owner::effects().windscreen_drops.work,
                        WindscreenDropWork::START,
                    )],
                    || {
                        // The point's three floats, as the buffer holds them at b[1].
                        let host = |n: usize| crate::pointers::host_at(b[1] + 4 * n).cast::<u32>();
                        let p: [PlainFloat; 3] = std::array::from_fn(|n| {
                            PlainFloat::from_bits(unsafe { host(n).read_unaligned() })
                        });
                        super::point_turned_x87(
                            &crate::effects::owner::effects().windscreen_drops,
                            &p,
                        );
                        for (n, cell) in p.iter().enumerate() {
                            unsafe { host(n).write_unaligned(cell.bits()) };
                        }
                    },
                );
            },
        );
    }
}
