//! `effects::draw_plane` (0x4C8F64): a quad laid on the windscreen drops' frame (a reading: the
//! ground glare): the rate, the place and two axes from the settings (+0x2720 .., +0x272C ..,
//! +0x2708 .., +0x2740), their lengths' shares, made one long and square
//! (`scene::normalise_vector3`, `scene::cross_product3`), scaled (`effects::scale_axes`);
//! nothing more unless the place's way faces the third axis away, or unless a frame switch is
//! set; `scene::depth_test_off`, render states 0x11 = 2, 0x12 = 2, `scene::set_texture` (the
//! effects' texture); the four corners; the texture places from the quad's corner in the mark
//! corners table, then from the plane's `places`; colour the view's light; unless a corner's
//! depth is at or below the nearest depth (signed compares of the bits): the corners
//! projected, their depth (plus the kind float and the plane's `lifts`, times the depth scale)
//! and rhw (1 over the depth times `rhw_k`), `scene::draw_fan`, `scene::depth_test_on`.
//!
//! Works on the effects settings.
//!
//! Caller: `effects::draw_overlays` The x87 runs are plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables; the vector helpers and
//! `scale_axes` their Rust versions. Refuses null settings and a corner past the mark corners table,
//! before anything.

use crate::effects::cell_x87::CellX87;
use crate::effects::kind_store::QuadWork;
use crate::effects::plain::ldi_cell;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::effects_settings_store::EffectsSettings;
use crate::scene::cross_product3::cross;
use crate::scene::vector3::normalise;

/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

/// The plane's constants (IDA 0x934F30 .. 0x934F7C). The offsets are from its start.
#[repr(C)]
pub struct PlaneTuning {
    /// +0x0: what the settings' rate is multiplied by into the first axis' scale
    pub rate_k: f32,
    /// +0x4: what the two axes' lengths are divided by
    pub shares: [f32; 2],
    /// +0xC: the four corners' texture places (u, v)
    pub places: [[f32; 2]; 4],
    /// +0x2C: what each corner's depth is raised by
    pub lifts: [f32; 4],
    /// +0x3C: what one over each corner's depth is multiplied by
    pub rhw_k: [f32; 4],
}

/// The plane's constants; the first values are the image's.
pub static PLANE_TUNING: PlaneTuning = PlaneTuning {
    rate_k: 1.0_f32,
    shares: [2048.0_f32, 1024.0_f32],
    places: [
        [0.0_f32, 0.5019531_f32],
        [0.9999_f32, 0.5019531_f32],
        [0.9999_f32; 2],
        [0.0_f32, 0.9999_f32],
    ],
    lifts: [500.0_f32; 4],
    rhw_k: [256.0_f32; 4],
};

#[cfg(test)]
impl PlaneTuning {
    pub const START: usize = 0x934F30;
}

/// The rate, the place and the first axis from the settings, and the first axis's length over
/// its divisor into the first of the flakes' `lengths`.
fn plane_axes(fl: &WindscreenFlakes, m: &EffectsSettings) {
    plane_axes_x87(m, &fl.timing, &fl.tables, &PLANE_TUNING);
}

/// The second axis, its length over its divisor into the second of the flakes' `lengths`.
fn plane_up(fl: &WindscreenFlakes, m: &EffectsSettings) {
    plane_up_x87(m, &fl.tables, &PLANE_TUNING);
}

/// The place as floats into the drops' `place`.
fn plane_place(drops: &WindscreenDrops, m: &EffectsSettings) {
    plane_place_x87(drops, m);
}

/// The place's way dotted with the third axis into the drops' `facing`.
fn plane_facing(drops: &WindscreenDrops, fl: &WindscreenFlakes) {
    plane_facing_x87(drops, &fl.tables);
}

/// The plane's centre and corners from the axes.
fn plane_corners(fl: &WindscreenFlakes, effects: &crate::effects::owner::Effects) {
    plane_corners_x87(&fl.tables, &effects.kinds.plane_quad);
}

/// The four corners projected.
fn plane_projected(effects: &crate::effects::owner::Effects) {
    plane_projected_x87(&effects.kinds.plane_quad);
}

pub fn draw_plane(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let buffer = ctx.part.vertices;
    let (race, scene) = (ctx.race, ctx.scene);
    use crate::scene::set_render_state::set_render_state;
    let m = effects.settings()?;
    let corner: u32 = effects.kinds.plane_quad.corner.get();
    plane_axes(fl, m);
    let [a0, a1, a2] = [0, 1, 2].map(|n| &fl.tables.axes[n]);
    normalise(a0);
    plane_up(fl, m);
    normalise(a1);
    cross(a1, a0, a2);
    normalise(a2);
    cross(a0, a2, a1);
    crate::effects::scale_axes::scale_axes(fl);
    plane_place(drops, m);
    normalise(&drops.hex.place);
    plane_facing(drops, fl);
    if !drops.state.facing.get().is_sign_negative() {
        return Ok(());
    }
    if race.frame.frame_work.lead_kept[7].get() == 0 {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a plane corner past g_marksCorners").into());
    }
    crate::scene::depth_test_off::depth_test_off(ctx.part);
    for state in [0x11, 0x12] {
        set_render_state(ctx.part, 2, state);
    }
    crate::scene::set_texture::set_texture(
        ctx.part,
        scene,
        scene.textures.kinds.kind_first_slots[202].get(),
    );
    plane_corners(fl, effects);
    let entry = &effects.blocks.corners.corners[corner as usize];
    let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
    let uv_k = effects.frame.draw_work.uv_k.get();
    for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
        let vertex = &buffer.vertices[n];
        vertex.u.set(x87::int_times(word(u) as i32, uv_k));
        vertex.v.set(x87::int_times(word(v) as i32, uv_k));
    }
    for n in 0..4 {
        let vertex = &buffer.vertices[n];
        vertex.u.set(x87::loaded(PLANE_TUNING.places[n][0]));
        vertex.v.set(x87::loaded(PLANE_TUNING.places[n][1]));
    }
    let light: u32 = ctx.view.light.view_light.get();
    for n in 0..4 {
        buffer.vertices[n].colour.set(light);
    }
    for n in 0..4 {
        if x87::bits_at_or_below(
            effects.kinds.plane_quad.corners[n][2].get(),
            race.course.numbers.k4096_a.get(),
        ) {
            return Ok(());
        }
    }
    plane_projected(effects);
    let [kind, z_k] = [
        effects.setup.kind_floats.kind_float.get(),
        effects.frame.draw_work.z_k.get(),
    ];
    for n in 0..4 {
        let c = &effects.kinds.plane_quad.corners[n];
        let vertex = &buffer.vertices[n];
        vertex.x.set(c[0].get());
        vertex.y.set(c[1].get());
        let depth = c[2].get();
        vertex.z.set(bh::stf(
            (bh::ld(depth) + bh::ld(kind) + bh::ld(PLANE_TUNING.lifts[n])) * bh::ld(z_k),
        ));
    }
    for n in 0..4 {
        let depth = effects.kinds.plane_quad.corners[n][2].get();
        buffer.vertices[n]
            .rhw
            .set(x87::reciprocal_times(depth, PLANE_TUNING.rhw_k[n]));
    }
    crate::effects::clip_quad::draw_fan(buffer, scene);
    crate::scene::depth_test_on::depth_test_on(ctx.part);
    Ok(())
}

use crate::effects::plain::{ld_cell, st_cell};
use crate::effects::windscreen_flakes_store::{WindscreenFlakeTables, WindscreenFlakeTiming};
use crate::math::x87_float as bh;

/// `plane_axes`'s x87 block as plain Rust on the x87 model (29 instructions), same order and memory places.
fn plane_axes_x87(
    m: &EffectsSettings,
    sti: &WindscreenFlakeTiming,
    sq: &WindscreenFlakeTables,
    pt: &PlaneTuning,
) {
    let t0 = ldi_cell(&m.windscreen_flakes_rate_factor);
    let t1 = t0 * bh::ld(pt.rate_k);
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
    let t20 = t19 / bh::ld(pt.shares[0]);
    sq.lengths[0].st(t20);
}

/// `plane_up`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
fn plane_up_x87(m: &EffectsSettings, sq: &WindscreenFlakeTables, pt: &PlaneTuning) {
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
    let t15 = t14 / bh::ld(pt.shares[1]);
    sq.lengths[1].st(t15);
}

/// `plane_place`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
fn plane_place_x87(drops: &WindscreenDrops, m: &EffectsSettings) {
    let t0 = ldi_cell(&m.light_point_b_negated[0]);
    st_cell(&drops.hex.place[0], t0);
    let t1 = ldi_cell(&m.light_point_b_negated[1]);
    st_cell(&drops.hex.place[1], t1);
    let t2 = ldi_cell(&m.light_point_b_negated[2]);
    st_cell(&drops.hex.place[2], t2);
}

/// `plane_facing`'s x87 block as plain Rust on the x87 model (9 instructions), same order and memory places.
fn plane_facing_x87(drops: &WindscreenDrops, sq: &WindscreenFlakeTables) {
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

/// `plane_corners`'s x87 block as plain Rust on the x87 model (47 instructions), same order and memory places.
fn plane_corners_x87(sq: &WindscreenFlakeTables, q: &QuadWork) {
    let t0 = bh::X(0.0);
    let t1 = bh::X(0.0);
    let t2 = t1 * sq.lengths[1].ld();
    let t3 = t0 * sq.lengths[0].ld();
    let t4 = t3 * sq.axes[0][0].ld();
    let t5 = t2 * sq.axes[1][0].ld();
    let t6 = t4 + t5;
    let t7 = t6 + sq.place[0].ld();
    q.corners[3][0].st(t7);
    let t8 = t3 * sq.axes[0][1].ld();
    let t9 = t2 * sq.axes[1][1].ld();
    let t10 = t8 + t9;
    let t11 = t10 + sq.place[1].ld();
    q.corners[3][1].st(t11);
    let t12 = t2 * sq.axes[1][2].ld();
    let t13 = t3 * sq.axes[0][2].ld();
    let t14 = t12 + t13;
    let t15 = t14 + sq.place[2].ld();
    q.corners[3][2].st(t15);
    let t16 = q.corners[3][0].ld();
    let t17 = t16 + sq.scaled_axes[1][0].ld();
    q.corners[0][0].st(t17);
    let t18 = t17 + sq.scaled_axes[0][0].ld();
    q.corners[1][0].st(t18);
    let t19 = t18 - sq.scaled_axes[1][0].ld();
    q.corners[2][0].st(t19);
    let t20 = q.corners[3][1].ld();
    let t21 = t20 + sq.scaled_axes[1][1].ld();
    q.corners[0][1].st(t21);
    let t22 = t21 + sq.scaled_axes[0][1].ld();
    q.corners[1][1].st(t22);
    let t23 = t22 - sq.scaled_axes[1][1].ld();
    q.corners[2][1].st(t23);
    let t24 = q.corners[3][2].ld();
    let t25 = t24 + sq.scaled_axes[1][2].ld();
    q.corners[0][2].st(t25);
    let t26 = t25 + sq.scaled_axes[0][2].ld();
    q.corners[1][2].st(t26);
    let t27 = t26 - sq.scaled_axes[1][2].ld();
    q.corners[2][2].st(t27);
}

/// `plane_projected`'s x87 block as plain Rust on the x87 model (40 instructions), same order and memory places.
fn plane_projected_x87(q: &QuadWork) {
    let t0 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    );
    let t1 = t0 / q.corners[0][2].ld();
    let t2 = t1 * q.corners[0][0].ld();
    let t3 = t2
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    q.corners[0][0].st(t3);
    let t4 = t1 * q.corners[0][1].ld();
    let t5 = -t4;
    let t6 = t5
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    q.corners[0][1].st(t6);
    let t7 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    );
    let t8 = t7 / q.corners[1][2].ld();
    let t9 = t8 * q.corners[1][0].ld();
    let t10 = t9
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    q.corners[1][0].st(t10);
    let t11 = t8 * q.corners[1][1].ld();
    let t12 = -t11;
    let t13 = t12
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    q.corners[1][1].st(t13);
    let t14 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    );
    let t15 = t14 / q.corners[2][2].ld();
    let t16 = t15 * q.corners[2][0].ld();
    let t17 = t16
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    q.corners[2][0].st(t17);
    let t18 = t15 * q.corners[2][1].ld();
    let t19 = -t18;
    let t20 = t19
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    q.corners[2][1].st(t20);
    let t21 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    );
    let t22 = t21 / q.corners[3][2].ld();
    let t23 = t22 * q.corners[3][0].ld();
    let t24 = t23
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    q.corners[3][0].st(t24);
    let t25 = t22 * q.corners[3][1].ld();
    let t26 = -t25;
    let t27 = t26
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    q.corners[3][1].st(t27);
}

#[cfg(test)]
mod reference_vectors {
    use super::{PlaneTuning, QuadWork};
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::with_copy;
    use crate::effects::asm_test::with_settings;
    use crate::effects::asm_test::{Op, Spec, recorded_writes};
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::windscreen_drop_state::WindscreenDropState;
    use crate::effects::windscreen_drop_vars::WindscreenDropVars;
    use crate::effects::windscreen_flakes_store::{WindscreenFlakeTables, WindscreenFlakeTiming};

    #[test]
    fn plane_axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137AF0,
                    hi: 0x1D003C,
                    reads: &[
                        (0x137FDA, F32),
                        (0x137FDE, F32),
                        (0x137FE2, F32),
                        (0x1CFFF0, F32),
                        (0x1CFFF4, F32),
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
        recorded_writes("effects.draw_plane.plane_axes_x87", &SPEC, |b| {
            with_copy::<WindscreenFlakeTiming, _>(b[0], WindscreenFlakeTiming::START, |sti| {
                with_copy::<WindscreenFlakeTables, _>(b[0], WindscreenFlakeTables::START, |sq| {
                    with_copy::<PlaneTuning, _>(b[0], PlaneTuning::START, |pt| {
                        with_settings(b[1], |m| super::plane_axes_x87(m, sti, sq, pt));
                    });
                });
            });
        });
    }

    #[test]
    fn plane_up_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137B8A,
                    hi: 0x1D003C,
                    reads: &[
                        (0x137FE6, F32),
                        (0x137FEA, F32),
                        (0x137FEE, F32),
                        (0x1CFFF8, F32),
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
        recorded_writes("effects.draw_plane.plane_up_x87", &SPEC, |b| {
            with_copy::<WindscreenFlakeTables, _>(b[0], WindscreenFlakeTables::START, |sq| {
                with_copy::<PlaneTuning, _>(b[0], PlaneTuning::START, |pt| {
                    with_settings(b[1], |m| super::plane_up_x87(m, sq, pt));
                });
            });
        });
    }

    #[test]
    fn plane_place_x87_matches() {
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
        recorded_writes("effects.draw_plane.plane_place_x87", &SPEC, |b| {
            with_overlays(
                b[0],
                &[overlay(
                    &crate::effects::owner::effects().windscreen_drops.hex,
                    WindscreenDropVars::START,
                )],
                || {
                    with_settings(b[1], |m| {
                        super::plane_place_x87(
                            &crate::effects::owner::effects().windscreen_drops,
                            m,
                        );
                    });
                },
            );
        });
    }

    #[test]
    fn plane_facing_x87_matches() {
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
        recorded_writes("effects.draw_plane.plane_facing_x87", &SPEC, |b| {
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
                            super::plane_facing_x87(
                                &crate::effects::owner::effects().windscreen_drops,
                                m,
                            );
                        },
                    );
                },
            );
        });
    }

    #[test]
    fn plane_corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x137B8A,
                hi: 0x1CFD88,
                reads: &[
                    (0x137FAE, F32),
                    (0x137FB2, F32),
                    (0x137FB6, F32),
                    (0x137FBA, F32),
                    (0x137FBE, F32),
                    (0x137FDA, F32),
                    (0x137FDE, F32),
                    (0x137FE2, F32),
                    (0x137FE6, F32),
                    (0x137FEA, F32),
                    (0x137FEE, F32),
                    (0x13802E, F32),
                    (0x138032, F32),
                    (0x138036, F32),
                    (0x13803A, F32),
                    (0x13803E, F32),
                    (0x138042, F32),
                    (0x1CFD7C, F32),
                    (0x1CFD80, F32),
                    (0x1CFD84, F32),
                ],
                writes: &[
                    (0x1CFD58, F32),
                    (0x1CFD5C, F32),
                    (0x1CFD60, F32),
                    (0x1CFD64, F32),
                    (0x1CFD68, F32),
                    (0x1CFD6C, F32),
                    (0x1CFD70, F32),
                    (0x1CFD74, F32),
                    (0x1CFD78, F32),
                    (0x1CFD7C, F32),
                    (0x1CFD80, F32),
                    (0x1CFD84, F32),
                ],
            }],
            sincos: false,
        };
        recorded_writes("effects.draw_plane.plane_corners_x87", &SPEC, |b| {
            with_copy::<WindscreenFlakeTables, _>(b[0], WindscreenFlakeTables::START, |sq| {
                with_copy::<QuadWork, _>(b[0], 0x934BD0, |q| super::plane_corners_x87(sq, q));
            });
        });
    }

    #[test]
    fn plane_projected_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x1CFC90,
                    hi: 0x1CFD88,
                    reads: &[
                        (0x1CFD58, F32),
                        (0x1CFD5C, F32),
                        (0x1CFD60, F32),
                        (0x1CFD64, F32),
                        (0x1CFD68, F32),
                        (0x1CFD6C, F32),
                        (0x1CFD70, F32),
                        (0x1CFD74, F32),
                        (0x1CFD78, F32),
                        (0x1CFD7C, F32),
                        (0x1CFD80, F32),
                        (0x1CFD84, F32),
                    ],
                    writes: &[
                        (0x1CFD58, F32),
                        (0x1CFD5C, F32),
                        (0x1CFD64, F32),
                        (0x1CFD68, F32),
                        (0x1CFD70, F32),
                        (0x1CFD74, F32),
                        (0x1CFD7C, F32),
                        (0x1CFD80, F32),
                    ],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_plane.plane_projected_x87", &SPEC, |b| {
            with_copy::<QuadWork, _>(b[1], 0x934BD0, |q| {
                super::plane_projected_x87(q);
            });
        });
    }
}
