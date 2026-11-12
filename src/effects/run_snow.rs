//! `effects::run_snow` (0x4C25F6): the snow's frame. `kept` is the settings' +0x58; the
//! light's place (`light_source`) turned by the view's place and the view matrix into
//! `light_place`, its way (`light_source_way`) turned without the places into `light_way`;
//! `effects::make_fade_colours`. `flakes` is the settings' +0x2760 (the flakes; none: `flag`
//! 0 and back), held to 0x800; the settings' +0x28 .. +0x48 into `view_matrix`, +0x1C .. +0x24
//! into `view_place`; `effects::place_snow_view`, `scene::set_texture` (the snow's texture),
//! `scene::depth_buffer_on`, `scene::depth_test_off`, render states 0x15 = 4, 0x13 = 5,
//! 0x14 = 6, 0x1C = 0, 0x1B = 1; the four vertices' rhw (`rhw`), colour the view's light |
//! 0xFF000000, specular 0, texture places from `uv`; the polygon's vertex count 4; 1 over
//! the farthest depth into `inverses` and `inverse`; when the settings' +0x10 is set the
//! fall (`fall`) gains the clock's step (since `last_clock`) times `fall_k` (rounded), kept
//! to 28 bits, `last_clock` the clock; the turn's nine floats and the focal length each four
//! times into `wide_turn` and `wide_focal`; `effects::draw_snow`; `scene::depth_test_on`.
//!
//! Works on the effects settings.
//!
//! Caller: `effects::run_frame`. The two turns are plain Rust on the x87 model (`math::x87_float`), in the original's order,
//! over the game's variables. Refuses null settings, before anything.

use crate::effects::cell_x87::CellX87;
use crate::effects::rain::Weather;
use crate::effects::snow::SnowLight;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;

const MOST: i32 = 0x800;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The light's place and way turned by the view.
fn turn_light(frame: &Frame, view_matrix: &[crate::session::Float; 16], weather: &Weather) {
    turn_light_x87(frame, view_matrix, &weather.snow_light);
}

pub fn run_snow(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    race: &Race,
    weather: &Weather,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    let part = crate::workers::game_scratch();
    let frame = &race.frame;
    use crate::scene::set_render_state::set_render_state;
    let (s, l) = (&weather.snow, &weather.snow_light);
    {
        let marks = effects.settings()?;
        l.kept.set(marks.snow_light_words[1].get());
        turn_light(frame, view.view_matrix, weather);
        crate::effects::make_fade_colours::make_fade_colours(view.light, weather, effects);
        let mut flakes = marks.snow_flakes.get() as i32;
        s.flakes.set(flakes as u32);
        if flakes == 0 {
            s.flag.set(0);
            return Ok(());
        }
        if flakes > MOST {
            flakes = MOST;
            s.flakes.set(flakes as u32);
        }
        for n in 0..9 {
            s.view_matrix[n].set(marks.listener_turn[n].get());
        }
        for n in 0..3 {
            s.view_place[n].set(marks.listener_place[n].get());
        }
        crate::effects::place_snow_view::place_snow_view(weather)?;
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.kinds.kind_first_slots[202].get(),
        );
        crate::scene::depth_buffer_on::depth_buffer(part, true);
        crate::scene::depth_test_off::depth_test_off(part);
        for (value, state) in STATES {
            set_render_state(part, value, state);
        }
        let rhw = l.rhw.get();
        let light = view.light | 0xFF00_0000;
        let [u0, v0, u1, v1] = l.uv.each_ref().map(crate::session::PlainFloat::get);
        let uv = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)];
        for (n, (u, v)) in uv.into_iter().enumerate() {
            let at = &buffer.vertices[n];
            at.rhw.set(rhw);
            at.colour.set(light);
            at.specular.set(0u32);
            at.u.set(u);
            at.v.set(v);
        }
        crate::scene::polygon_work::front_end()
            .polygon_vertex_count
            .set(4);
        let inverse = x87::reciprocal(&crate::scene::polygon_work::front_end().window.farthest);
        for r in &s.inverses {
            r.set(inverse);
        }
        s.inverse.set(inverse);
        let marks = effects.settings()?;
        if marks.other_marks_shown.get() != 0 {
            let ticks = race
                .shared
                .physics_tables
                .scene_clock
                .get()
                .wrapping_sub(s.last_clock.get());
            let fall = x87::int_times_plus_int_to_int(ticks as i32, &l.fall_k, s.fall.get() as i32);
            s.fall.set(fall as u32);
        }
        s.fall.set(s.fall.get() & 0x0FFF_FFFF);
        s.last_clock
            .set(race.shared.physics_tables.scene_clock.get());
        for (n, v) in s.matrix_copy.iter().enumerate() {
            for m in 0..4 {
                s.wide_turn[n][m].set(v.get());
            }
        }
        let focal = crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get();
        for m in 0..4 {
            s.wide_focal[m].set(focal);
        }
        crate::effects::draw_snow::draw_snow(buffer, weather, scene, effects)?;
        crate::scene::depth_test_on::depth_test_on(part);
    }
    Ok(())
}

/// `turn_light`'s x87 block as plain Rust on the x87 model (126 instructions), same order and memory places.
fn turn_light_x87(frame: &Frame, view_matrix: &[crate::session::Float; 16], l: &SnowLight) {
    let t0 = l.light_source[0].ld();
    let t1 = l.light_source[1].ld();
    let t2 = l.light_source[2].ld();
    let t3 = frame.matrices.view_place[0].ld();
    let t4 = t3 * t2;
    let t5 = frame.matrices.view_place[4].ld();
    let t6 = t5 * t1;
    let t7 = frame.matrices.view_place[8].ld();
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = t10 + frame.matrices.view_place[12].ld();
    let t12 = frame.matrices.view_place[1].ld();
    let t13 = t12 * t2;
    let t14 = frame.matrices.view_place[5].ld();
    let t15 = t14 * t1;
    let t16 = frame.matrices.view_place[9].ld();
    let t17 = t16 * t0;
    let t18 = t15 + t17;
    let t19 = t13 + t18;
    let t20 = t19 + frame.matrices.view_place[13].ld();
    let t21 = frame.matrices.view_place[2].ld();
    let t22 = t2 * t21;
    let t23 = frame.matrices.view_place[6].ld();
    let t24 = t1 * t23;
    let t25 = frame.matrices.view_place[10].ld();
    let t26 = t0 * t25;
    let t27 = t26 + t24;
    let t28 = t22 + t27;
    let t29 = t28 + frame.matrices.view_place[14].ld();
    let t30 = view_matrix[0].ld();
    let t31 = t30 * t11;
    let t32 = view_matrix[4].ld();
    let t33 = t32 * t20;
    let t34 = view_matrix[8].ld();
    let t35 = t34 * t29;
    let t36 = t33 + t35;
    let t37 = t31 + t36;
    let t38 = t37 + view_matrix[12].ld();
    let t39 = view_matrix[1].ld();
    let t40 = t39 * t11;
    let t41 = view_matrix[5].ld();
    let t42 = t41 * t20;
    let t43 = view_matrix[9].ld();
    let t44 = t43 * t29;
    let t45 = t42 + t44;
    let t46 = t40 + t45;
    let t47 = t46 + view_matrix[13].ld();
    let t48 = view_matrix[2].ld();
    let t49 = t11 * t48;
    let t50 = view_matrix[6].ld();
    let t51 = t20 * t50;
    let t52 = view_matrix[10].ld();
    let t53 = t29 * t52;
    let t54 = t53 + t51;
    let t55 = t49 + t54;
    let t56 = t55 + view_matrix[14].ld();
    l.light_place[0].st(t38);
    l.light_place[1].st(t47);
    l.light_place[2].st(t56);
    let t57 = l.light_source_way[0].ld();
    let t58 = l.light_source_way[1].ld();
    let t59 = l.light_source_way[2].ld();
    let t60 = frame.matrices.view_place[0].ld();
    let t61 = t60 * t59;
    let t62 = frame.matrices.view_place[4].ld();
    let t63 = t62 * t58;
    let t64 = frame.matrices.view_place[8].ld();
    let t65 = t64 * t57;
    let t66 = t63 + t65;
    let t67 = t61 + t66;
    let t68 = frame.matrices.view_place[1].ld();
    let t69 = t68 * t59;
    let t70 = frame.matrices.view_place[5].ld();
    let t71 = t70 * t58;
    let t72 = frame.matrices.view_place[9].ld();
    let t73 = t72 * t57;
    let t74 = t71 + t73;
    let t75 = t69 + t74;
    let t76 = frame.matrices.view_place[2].ld();
    let t77 = t59 * t76;
    let t78 = frame.matrices.view_place[6].ld();
    let t79 = t58 * t78;
    let t80 = frame.matrices.view_place[10].ld();
    let t81 = t57 * t80;
    let t82 = t81 + t79;
    let t83 = t77 + t82;
    let t84 = view_matrix[0].ld();
    let t85 = t84 * t67;
    let t86 = view_matrix[4].ld();
    let t87 = t86 * t75;
    let t88 = view_matrix[8].ld();
    let t89 = t88 * t83;
    let t90 = t87 + t89;
    let t91 = t85 + t90;
    let t92 = view_matrix[1].ld();
    let t93 = t92 * t67;
    let t94 = view_matrix[5].ld();
    let t95 = t94 * t75;
    let t96 = view_matrix[9].ld();
    let t97 = t96 * t83;
    let t98 = t95 + t97;
    let t99 = t93 + t98;
    let t100 = view_matrix[2].ld();
    let t101 = t67 * t100;
    let t102 = view_matrix[6].ld();
    let t103 = t75 * t102;
    let t104 = view_matrix[10].ld();
    let t105 = t83 * t104;
    let t106 = t105 + t103;
    let t107 = t101 + t106;
    l.light_way[0].st(t91);
    l.light_way[1].st(t99);
    l.light_way[2].st(t107);
}

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::with_copy;
    use crate::effects::asm_test::{Op, Spec, recorded_writes};
    use crate::effects::snow::SnowLight;
    use crate::race::owner::race;

    #[test]
    fn turn_light_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[
                        (0x1EA2CC, F32),
                        (0x1EA2D0, F32),
                        (0x1EA2D4, F32),
                        (0x1EA2DC, F32),
                        (0x1EA2E0, F32),
                        (0x1EA2E4, F32),
                        (0x1EA2EC, F32),
                        (0x1EA2F0, F32),
                        (0x1EA2F4, F32),
                        (0x1EA2FC, F32),
                        (0x1EA300, F32),
                        (0x1EA304, F32),
                        (0x1EA3CC, F32),
                        (0x1EA3D0, F32),
                        (0x1EA3D4, F32),
                        (0x1EA3DC, F32),
                        (0x1EA3E0, F32),
                        (0x1EA3E4, F32),
                        (0x1EA3EC, F32),
                        (0x1EA3F0, F32),
                        (0x1EA3F4, F32),
                        (0x1EA3FC, F32),
                        (0x1EA400, F32),
                        (0x1EA404, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x1764F0,
                    hi: 0x176618,
                    reads: &[
                        (0x1765CC, F32),
                        (0x1765D0, F32),
                        (0x1765D4, F32),
                        (0x1765D8, F32),
                        (0x1765DC, F32),
                        (0x1765E0, F32),
                    ],
                    writes: &[
                        (0x1764F0, F32),
                        (0x1764F4, F32),
                        (0x1764F8, F32),
                        (0x1764FC, F32),
                        (0x176500, F32),
                        (0x176504, F32),
                    ],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.run_snow.turn_light_x87", &SPEC, |b| {
            with_copy(b[1], SnowLight::START, |l| {
                super::turn_light_x87(&race().frame, &race().frame.matrices.view_matrix, l);
            });
        });
    }
}
