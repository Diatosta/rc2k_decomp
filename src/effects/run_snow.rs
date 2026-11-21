//! `effects::run_snow`: the snow's frame. The light's place and way are turned by the view into the
//! snow's light, then the fade colours are made. The flakes (the settings' `snow_flakes`, held to 0x800;
//! none leaves `flag` 0) are drawn by `effects::draw_snow` from the view's place and matrix (the
//! settings' `listener_place` and `listener_turn`), with the four vertices' rhw, the view's light as colour and the
//! texture places from `uv`. When the settings' `other_marks_shown` is set the fall gains the clock's step times
//! `fall_k`, kept to 28 bits.
//!
//! Works on the effects settings.
//!
//! The two turns are plain Rust on the x87 model (`math::x87_float`), over the game's variables.
//! Refuses null settings, before anything.

use crate::effects::rain_store::Weather;
use crate::effects::snow_store::SnowLight;
use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;

const MOST: i32 = 0x800;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The light's place and way turned by the view.
fn turn_light(
    frame: &Frame,
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    weather: &Weather,
) {
    turn_light_x87(frame, view_matrix, &mut weather.snow_light.borrow_mut());
}

pub fn run_snow(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    race: &Race,
    weather: &Weather,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    let part = scene.scratch.game_scratch();
    let frame = &race.frame;
    use crate::scene::set_render_state::set_render_state;
    let (snow, l) = (&weather.snow, &weather.snow_light);
    {
        let marks = effects.settings()?;
        l.borrow_mut().kept = marks.snow_light_words[1].get();
        turn_light(frame, view.view_matrix, weather);
        crate::effects::make_fade_colours::make_fade_colours(view.light, weather, effects);
        let mut flakes = marks.snow_flakes.get() as i32;
        snow.borrow_mut().flakes = flakes as u32;
        if flakes == 0 {
            snow.borrow_mut().flag = 0;
            return Ok(());
        }
        if flakes > MOST {
            flakes = MOST;
            snow.borrow_mut().flakes = flakes as u32;
        }
        {
            let mut s = snow.borrow_mut();
            for n in 0..9 {
                s.view_matrix[n] = marks.listener_turn[n].get();
            }
            for n in 0..3 {
                s.view_place[n] = marks.listener_place[n].get();
            }
        }
        crate::effects::place_snow_view::place_snow_view(weather);
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.kinds.borrow().kind_first_slots[202],
        );
        crate::scene::depth_buffer_on::depth_buffer(part, true);
        crate::scene::depth_test_off::depth_test_off(part);
        for (value, state) in STATES {
            set_render_state(part, value, state);
        }
        let rhw = l.borrow().rhw;
        let light = view.light | 0xFF00_0000;
        let [u0, v0, u1, v1] = l.borrow().uv;
        let uv = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)];
        for (n, (u, v)) in uv.into_iter().enumerate() {
            let at = &buffer.vertices[n];
            at.rhw.set(rhw);
            at.colour.set(light);
            at.specular.set(0u32);
            at.u.set(u);
            at.v.set(v);
        }
        scene
            .scratch
            .front_end_polygon()
            .polygon_vertex_count
            .set(4);
        let inverse = x87::reciprocal(&scene.scratch.front_end_polygon().window.farthest);
        snow.borrow_mut().inverses = [inverse; 4];
        snow.borrow_mut().inverse = inverse;
        let marks = effects.settings()?;
        let clock = race.shared.run().scene_clock;
        if marks.other_marks_shown.get() != 0 {
            let (last_clock, fall) = {
                let s = snow.borrow();
                (s.last_clock, s.fall)
            };
            let ticks = clock.wrapping_sub(last_clock);
            let fall = x87::int_times_plus_int_to_int(ticks as i32, l.borrow().fall_k, fall as i32);
            snow.borrow_mut().fall = fall as u32;
        }
        {
            let mut s = snow.borrow_mut();
            s.fall &= 0x0FFF_FFFF;
            s.last_clock = clock;
            s.wide_turn = s.matrix_copy.map(|v| [v; 4]);
            s.wide_focal = [scene.scratch.front_end_polygon().window.view_scale.get(); 4];
        }
        crate::effects::draw_snow::draw_snow(buffer, weather, scene, effects)?;
        crate::scene::depth_test_on::depth_test_on(part);
    }
    Ok(())
}

/// `turn_light`'s arithmetic as plain Rust on the x87 model, in the same order.
fn turn_light_x87(
    frame: &Frame,
    view_matrix: &[crate::memory::session::PlainFloat; 16],
    l: &mut SnowLight,
) {
    let t0 = bh::ld(l.light_source[0]);
    let t1 = bh::ld(l.light_source[1]);
    let t2 = bh::ld(l.light_source[2]);
    let t3 = frame.matrices.get().view_place[0].ld();
    let t4 = t3 * t2;
    let t5 = frame.matrices.get().view_place[4].ld();
    let t6 = t5 * t1;
    let t7 = frame.matrices.get().view_place[8].ld();
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = t10 + frame.matrices.get().view_place[12].ld();
    let t12 = frame.matrices.get().view_place[1].ld();
    let t13 = t12 * t2;
    let t14 = frame.matrices.get().view_place[5].ld();
    let t15 = t14 * t1;
    let t16 = frame.matrices.get().view_place[9].ld();
    let t17 = t16 * t0;
    let t18 = t15 + t17;
    let t19 = t13 + t18;
    let t20 = t19 + frame.matrices.get().view_place[13].ld();
    let t21 = frame.matrices.get().view_place[2].ld();
    let t22 = t2 * t21;
    let t23 = frame.matrices.get().view_place[6].ld();
    let t24 = t1 * t23;
    let t25 = frame.matrices.get().view_place[10].ld();
    let t26 = t0 * t25;
    let t27 = t26 + t24;
    let t28 = t22 + t27;
    let t29 = t28 + frame.matrices.get().view_place[14].ld();
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
    l.light_place[0] = f32::from_bits(bh::st(t38));
    l.light_place[1] = f32::from_bits(bh::st(t47));
    l.light_place[2] = f32::from_bits(bh::st(t56));
    let t57 = bh::ld(l.light_source_way[0]);
    let t58 = bh::ld(l.light_source_way[1]);
    let t59 = bh::ld(l.light_source_way[2]);
    let t60 = frame.matrices.get().view_place[0].ld();
    let t61 = t60 * t59;
    let t62 = frame.matrices.get().view_place[4].ld();
    let t63 = t62 * t58;
    let t64 = frame.matrices.get().view_place[8].ld();
    let t65 = t64 * t57;
    let t66 = t63 + t65;
    let t67 = t61 + t66;
    let t68 = frame.matrices.get().view_place[1].ld();
    let t69 = t68 * t59;
    let t70 = frame.matrices.get().view_place[5].ld();
    let t71 = t70 * t58;
    let t72 = frame.matrices.get().view_place[9].ld();
    let t73 = t72 * t57;
    let t74 = t71 + t73;
    let t75 = t69 + t74;
    let t76 = frame.matrices.get().view_place[2].ld();
    let t77 = t59 * t76;
    let t78 = frame.matrices.get().view_place[6].ld();
    let t79 = t58 * t78;
    let t80 = frame.matrices.get().view_place[10].ld();
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
    l.light_way[0] = f32::from_bits(bh::st(t91));
    l.light_way[1] = f32::from_bits(bh::st(t99));
    l.light_way[2] = f32::from_bits(bh::st(t107));
}

#[cfg(test)]
mod tests;
