//! `effects::draw_shadow`: the car's ground shadow (probably: the quad is drawn flat on the ground
//! under the car), unless the quad's `corner` is -1. The depth buffer and test are switched on
//! (`scene::depth_buffer_on`, `scene::depth_test_on`) with render states 0x15 = 4, 0x13 = 5, 0x14 =
//! 6, 0x1C = 0, 0x1B = 1. The quad's two `sides` are made one long and square
//! (`scene::cross_product3`, `scene::normalise_vector3`); its `matrix` is turned about z by an
//! angle from the settings' +0x26A4 (two lines, by the quad's `ratio`) and the light's way turned
//! by it; the quad's `corners` about its `place` (the `frame`) are drawn in a little and turned to
//! the view (`scene::rotate_point`); the texture places come from the mark corner the quad's
//! `corner` names and the colour is the shadow colour | 0xFF000000 brought halfway to white. Unless
//! a corner's depth is at or below the nearest depth (signed compares of the bits) the corners are
//! projected, their depths less the quad's lift times a depth scale, and drawn with
//! `scene::draw_fan`.
//!
//! The long x87 runs are plain Rust on the x87 model (`math::x87_float`); the vector helpers are
//! Rust too. Refuses, before anything, a corner past the mark corners table and an angle with no
//! sine and cosine.

use crate::effects::cell_x87::CellX87;
use crate::effects::kind_store::{CONSTS_1, CONSTS_2, KindConsts, QuadWork};
use crate::effects::mask_store::Masks;
use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::cross_product3::cross;
use crate::scene::matrix3::{Axis, extended, rounded_value, turn_matrix3, turnable};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::normalise;
use crate::session::PlainFloat;

/// A shadow's variables (the typed statics of `effects::kind_store`: the second shadow's,
/// `effects::draw_overlays`'s, are set 1's, the first's set 2's) and its x87 blocks.
pub struct Shadow<'a> {
    pub quad: &'a QuadWork,
    pub consts: &'a KindConsts,
    /// The quad is lifted along it.
    pub up: &'a PlainFloat,
    /// The settings' field the angle comes from.
    pub field: fn(&EffectsSettings) -> &PlainFloat,
}

/// `draw_shadow`'s.
pub fn first(effects: &Effects) -> Shadow<'_> {
    Shadow {
        quad: &effects.kinds.shadow_quad,
        consts: &CONSTS_2,
        up: &effects.kinds.shadow_extra.extra,
        field: |marks| &marks.engine_speed_scaled_copy,
    }
}

/// `effects::draw_overlays`'s.
pub fn second(effects: &Effects) -> Shadow<'_> {
    Shadow {
        quad: &effects.kinds.overlay_quad,
        consts: &CONSTS_1,
        up: &effects.kinds.overlay_extras.extra,
        field: |marks| &marks.engine_speed_copy,
    }
}

/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The z turn's angle (see the header), 80 bits.
fn angle(sh: &Shadow, effects: &crate::effects::owner::Effects) -> Result<[u8; 10], Refused> {
    Ok({
        let marks = effects.settings()?;
        let split = sh.quad.low.get();
        let v = (sh.field)(marks).get();
        if x87::bits_at_or_below(split, v) {
            x87::less_times_plus_negated_80(v, split, &sh.quad.slope, &sh.quad.middle)
        } else {
            x87::times_plus_negated_80(v, &sh.quad.step, &sh.quad.start)
        }
    })
}

pub fn draw_shadow(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    masks: &Masks,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    draw_shadow_with(ctx, masks, &first(effects), true, effects)
}

impl Shadow<'_> {
    /// The corner (-1: no shadow).
    pub fn corner(&self) -> u32 {
        self.quad.corner.get()
    }

    /// The angle's refusal, if any (for a caller that checks before drawing).
    pub fn angle_refuses(&self, effects: &crate::effects::owner::Effects) -> Option<Refused> {
        match angle(self, effects) {
            Err(e) => Some(e),
            Ok(a) => { turnable(&a) }.err(),
        }
    }
}

/// A shadow (see the header) on `sh`'s variables; `depth` the two depth calls first.
pub fn draw_shadow_with(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    masks: &Masks,
    sh: &Shadow,
    depth: bool,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let buffer = &ctx.part.vertices;
    let race = ctx.race;
    use crate::scene::set_render_state::set_render_state;
    let q = sh.quad;
    let corner: u32 = q.corner.get();
    if corner == 0xFFFF_FFFF {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a shadow corner past g_marksCorners").into());
    }
    effects.settings()?;
    turnable(&angle(sh, effects)?)?;
    let temps = [const { PlainFloat::new(0.0) }; 2];

    if depth {
        crate::scene::depth_buffer_on::depth_buffer(ctx.part, true);
        crate::scene::depth_test_on::depth_test_on(ctx.part);
    }
    for (value, state) in STATES {
        set_render_state(ctx.part, value, state);
    }
    for n in 0..2 {
        for k in 0..3 {
            q.frame[n][k].set(q.sides[n][k].get());
        }
    }
    let [s0, s1, s2] = [0, 1, 2].map(|n| &q.frame[n]);
    normalise(s0);
    normalise(s1);
    cross(s0, s1, s2);
    cross(s2, s0, s1);
    normalise(s1);
    normalise(s2);
    let a = angle(sh, effects)?;
    turn_matrix3(Axis::Z, &q.matrix, a)?;
    light_turned_x87(sh, &q.matrix, &temps);
    for n in 0..3 {
        for k in 0..3 {
            q.matrix[n + 3 * k].set(q.frame[n][k].get());
        }
    }
    second_turned_x87(sh, &q.matrix, &temps);
    cross(s2, s1, s0);
    cross(s2, s0, s1);
    corners_x87(sh);
    normalise(&q.way);
    corners_in_x87(sh);
    let view = &race.frame.matrices.get().listener_matrix;
    for item in q.corners.iter().take(4) {
        let p: [f32; 3] = std::array::from_fn(|k| item[k].get());
        let turned = rotate_point(view, p.map(extended));
        for (k, v) in turned.into_iter().enumerate() {
            item[k].set(rounded_value(v));
        }
    }
    let entry = &effects.blocks.corners.corners[corner as usize]; // under 0x100, checked before
    let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
    let uv_k = effects.frame.draw_work.uv_k.get();
    for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
        let vertex = &buffer.vertices[n];
        vertex.u.set(x87::int_times(word(u) as i32, uv_k));
        vertex.v.set(x87::int_times(word(v) as i32, uv_k));
    }
    let mut c = masks.surface.lives_by_level[8].get() | 0xFF00_0000;
    let half = |shift: u32| ((!(c >> shift) & 0xFF) >> 1) << shift;
    let (r, gr, bl) = (half(16), half(8), half(0));
    c = c.wrapping_add(r).wrapping_add(gr).wrapping_add(bl);
    for item in buffer.vertices.iter().take(4) {
        item.colour.set(c);
    }
    for item in q.corners.iter().take(4) {
        if x87::bits_at_or_below(item[2].get(), race.course.numbers.get().k4096_a.get()) {
            return Ok(());
        }
    }
    projected_x87(sh);
    let z_k = effects.frame.draw_work.z_k.get();
    for n in 0..4 {
        let vertex = &buffer.vertices[n];
        vertex.x.set(q.corners[n][0].get());
        vertex.y.set(q.corners[n][1].get());
        let depth = x87::sub_mul(&q.corners[n][2], sh.consts.lifts[n], z_k);
        vertex.z.set(depth);
    }
    crate::effects::clip_quad::draw_fan(buffer, ctx.scene);
    Ok(())
}

use crate::effects::plain::{ld_cell, st_cell};
use crate::math::x87_float as bh;
use crate::race::effects_settings_store::EffectsSettings;

/// `light_turned`'s arithmetic as plain Rust on the x87 model, in the same order, on either
/// shadow's variables.
fn light_turned_x87(s: &Shadow, m: &[PlainFloat; 9], t: &[PlainFloat; 2]) {
    let t0 = bh::ld(s.consts.light[0]);
    let t1 = bh::ld(s.consts.light[1]);
    let t2 = bh::ld(s.consts.light[2]);
    let t3 = t2 * ld_cell(&m[0]);
    let t4 = t1 * ld_cell(&m[1]);
    let t5 = t0 * ld_cell(&m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st_cell(&t[0], t7);
    let t8 = t2 * ld_cell(&m[3]);
    let t9 = t1 * ld_cell(&m[4]);
    let t10 = t0 * ld_cell(&m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st_cell(&t[1], t12);
    let t13 = t2 * ld_cell(&m[6]);
    let t14 = t1 * ld_cell(&m[7]);
    let t15 = t0 * ld_cell(&m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld_cell(&t[1]);
    let t19 = ld_cell(&t[0]);
    s.quad.light_turned[0].st(t19);
    s.quad.light_turned[1].st(t18);
    s.quad.light_turned[2].st(t17);
}

/// The second light turn's arithmetic as plain Rust on the x87 model, in the same order, on
/// either shadow's variables.
fn second_turned_x87(s: &Shadow, m: &[PlainFloat; 9], t: &[PlainFloat; 2]) {
    let t0 = s.quad.light_turned[2].ld();
    let t1 = s.quad.light_turned[1].ld();
    let t2 = s.quad.light_turned[0].ld();
    let t3 = t2 * ld_cell(&m[0]);
    let t4 = t1 * ld_cell(&m[1]);
    let t5 = t0 * ld_cell(&m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st_cell(&t[0], t7);
    let t8 = t2 * ld_cell(&m[3]);
    let t9 = t1 * ld_cell(&m[4]);
    let t10 = t0 * ld_cell(&m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st_cell(&t[1], t12);
    let t13 = t2 * ld_cell(&m[6]);
    let t14 = t1 * ld_cell(&m[7]);
    let t15 = t0 * ld_cell(&m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld_cell(&t[1]);
    let t19 = ld_cell(&t[0]);
    s.quad.frame[1][0].st(t19);
    s.quad.frame[1][1].st(t18);
    s.quad.frame[1][2].st(t17);
}

/// `corners`'s arithmetic as plain Rust on the x87 model, in the same order, on either shadow's
/// variables.
fn corners_x87(s: &Shadow) {
    let t0 = s.quad.frame[0][0].ld();
    let t1 = t0 * s.quad.ratio.ld();
    s.quad.frame[0][0].st(t1);
    let t2 = s.quad.frame[0][1].ld();
    let t3 = t2 * s.quad.ratio.ld();
    s.quad.frame[0][1].st(t3);
    let t4 = s.quad.frame[0][2].ld();
    let t5 = t4 * s.quad.ratio.ld();
    s.quad.frame[0][2].st(t5);
    let t6 = s.quad.frame[1][0].ld();
    let t7 = t6 * s.quad.size.ld();
    s.quad.frame[1][0].st(t7);
    let t8 = s.quad.frame[1][1].ld();
    let t9 = t8 * s.quad.size.ld();
    s.quad.frame[1][1].st(t9);
    let t10 = s.quad.frame[1][2].ld();
    let t11 = t10 * s.quad.size.ld();
    s.quad.frame[1][2].st(t11);
    let t12 = s.quad.place[0].ld();
    let t13 = t12 - s.quad.frame[0][0].ld();
    s.quad.corners[0][0].st(t13);
    let t14 = s.quad.place[1].ld();
    let t15 = t14 - s.quad.frame[0][1].ld();
    s.quad.corners[0][1].st(t15);
    let t16 = s.quad.place[2].ld();
    let t17 = t16 - s.quad.frame[0][2].ld();
    s.quad.corners[0][2].st(t17);
    let t18 = s.quad.place[0].ld();
    let t19 = t18 + s.quad.frame[0][0].ld();
    s.quad.corners[1][0].st(t19);
    let t20 = s.quad.place[1].ld();
    let t21 = t20 + s.quad.frame[0][1].ld();
    s.quad.corners[1][1].st(t21);
    let t22 = s.quad.place[2].ld();
    let t23 = t22 + s.quad.frame[0][2].ld();
    s.quad.corners[1][2].st(t23);
    let t24 = s.quad.place[0].ld();
    let t25 = t24 + s.quad.frame[0][0].ld();
    let t26 = t25 + s.quad.frame[1][0].ld();
    s.quad.corners[2][0].st(t26);
    let t27 = s.quad.place[1].ld();
    let t28 = t27 + s.quad.frame[0][1].ld();
    let t29 = t28 + s.quad.frame[1][1].ld();
    s.quad.corners[2][1].st(t29);
    let t30 = s.quad.place[2].ld();
    let t31 = t30 + s.quad.frame[0][2].ld();
    let t32 = t31 + s.quad.frame[1][2].ld();
    s.quad.corners[2][2].st(t32);
    let t33 = s.quad.place[0].ld();
    let t34 = t33 - s.quad.frame[0][0].ld();
    let t35 = t34 + s.quad.frame[1][0].ld();
    s.quad.corners[3][0].st(t35);
    let t36 = s.quad.place[1].ld();
    let t37 = t36 - s.quad.frame[0][1].ld();
    let t38 = t37 + s.quad.frame[1][1].ld();
    s.quad.corners[3][1].st(t38);
    let t39 = s.quad.place[2].ld();
    let t40 = t39 - s.quad.frame[0][2].ld();
    let t41 = t40 + s.quad.frame[1][2].ld();
    s.quad.corners[3][2].st(t41);
    let t42 = s.quad.matrix[1].ld();
    let t43 = t42 * s.up.ld();
    let t44 = t43 + s.quad.corners[0][0].ld();
    s.quad.corners[0][0].st(t44);
    let t45 = s.quad.matrix[4].ld();
    let t46 = t45 * s.up.ld();
    let t47 = t46 + s.quad.corners[0][1].ld();
    s.quad.corners[0][1].st(t47);
    let t48 = s.quad.matrix[7].ld();
    let t49 = t48 * s.up.ld();
    let t50 = t49 + s.quad.corners[0][2].ld();
    s.quad.corners[0][2].st(t50);
    let t51 = s.quad.matrix[1].ld();
    let t52 = t51 * s.up.ld();
    let t53 = t52 + s.quad.corners[1][0].ld();
    s.quad.corners[1][0].st(t53);
    let t54 = s.quad.matrix[4].ld();
    let t55 = t54 * s.up.ld();
    let t56 = t55 + s.quad.corners[1][1].ld();
    s.quad.corners[1][1].st(t56);
    let t57 = s.quad.matrix[7].ld();
    let t58 = t57 * s.up.ld();
    let t59 = t58 + s.quad.corners[1][2].ld();
    s.quad.corners[1][2].st(t59);
    let t60 = s.quad.matrix[1].ld();
    let t61 = t60 * s.up.ld();
    let t62 = t61 + s.quad.corners[2][0].ld();
    s.quad.corners[2][0].st(t62);
    let t63 = s.quad.matrix[4].ld();
    let t64 = t63 * s.up.ld();
    let t65 = t64 + s.quad.corners[2][1].ld();
    s.quad.corners[2][1].st(t65);
    let t66 = s.quad.matrix[7].ld();
    let t67 = t66 * s.up.ld();
    let t68 = t67 + s.quad.corners[2][2].ld();
    s.quad.corners[2][2].st(t68);
    let t69 = s.quad.matrix[1].ld();
    let t70 = t69 * s.up.ld();
    let t71 = t70 + s.quad.corners[3][0].ld();
    s.quad.corners[3][0].st(t71);
    let t72 = s.quad.matrix[4].ld();
    let t73 = t72 * s.up.ld();
    let t74 = t73 + s.quad.corners[3][1].ld();
    s.quad.corners[3][1].st(t74);
    let t75 = s.quad.matrix[7].ld();
    let t76 = t75 * s.up.ld();
    let t77 = t76 + s.quad.corners[3][2].ld();
    s.quad.corners[3][2].st(t77);
    let t78 = s.quad.corners[2][0].ld();
    let t79 = t78 - s.quad.corners[1][0].ld();
    s.quad.way[0].st(t79);
    let t80 = s.quad.corners[2][1].ld();
    let t81 = t80 - s.quad.corners[1][1].ld();
    s.quad.way[1].st(t81);
    let t82 = s.quad.corners[2][2].ld();
    let t83 = t82 - s.quad.corners[1][2].ld();
    s.quad.way[2].st(t83);
}

/// The arithmetic on the corners, after the way is normalised, as plain Rust on the x87 model, in
/// the same order, on either shadow's variables.
fn corners_in_x87(s: &Shadow) {
    let t0 = s.quad.way[0].ld();
    let t1 = t0 * s.quad.spread.ld();
    s.quad.way[0].st(t1);
    let t2 = s.quad.way[1].ld();
    let t3 = t2 * s.quad.spread.ld();
    s.quad.way[1].st(t3);
    let t4 = s.quad.way[2].ld();
    let t5 = t4 * s.quad.spread.ld();
    s.quad.way[2].st(t5);
    let t6 = s.quad.corners[0][0].ld();
    let t7 = t6 - s.quad.way[0].ld();
    s.quad.corners[0][0].st(t7);
    let t8 = s.quad.corners[0][1].ld();
    let t9 = t8 - s.quad.way[1].ld();
    s.quad.corners[0][1].st(t9);
    let t10 = s.quad.corners[0][2].ld();
    let t11 = t10 - s.quad.way[2].ld();
    s.quad.corners[0][2].st(t11);
    let t12 = s.quad.corners[1][0].ld();
    let t13 = t12 - s.quad.way[0].ld();
    s.quad.corners[1][0].st(t13);
    let t14 = s.quad.corners[1][1].ld();
    let t15 = t14 - s.quad.way[1].ld();
    s.quad.corners[1][1].st(t15);
    let t16 = s.quad.corners[1][2].ld();
    let t17 = t16 - s.quad.way[2].ld();
    s.quad.corners[1][2].st(t17);
    let t18 = s.quad.corners[2][0].ld();
    let t19 = t18 - s.quad.way[0].ld();
    s.quad.corners[2][0].st(t19);
    let t20 = s.quad.corners[2][1].ld();
    let t21 = t20 - s.quad.way[1].ld();
    s.quad.corners[2][1].st(t21);
    let t22 = s.quad.corners[2][2].ld();
    let t23 = t22 - s.quad.way[2].ld();
    s.quad.corners[2][2].st(t23);
    let t24 = s.quad.corners[3][0].ld();
    let t25 = t24 - s.quad.way[0].ld();
    s.quad.corners[3][0].st(t25);
    let t26 = s.quad.corners[3][1].ld();
    let t27 = t26 - s.quad.way[1].ld();
    s.quad.corners[3][1].st(t27);
    let t28 = s.quad.corners[3][2].ld();
    let t29 = t28 - s.quad.way[2].ld();
    s.quad.corners[3][2].st(t29);
}

/// `projected`'s arithmetic as plain Rust on the x87 model, in the same order, on either shadow's
/// variables.
fn projected_x87(s: &Shadow) {
    let t0 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    );
    let t1 = t0 / s.quad.corners[0][2].ld();
    let t2 = t1 * s.quad.corners[0][0].ld();
    let t3 = t2
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    s.quad.corners[0][0].st(t3);
    let t4 = t1 * s.quad.corners[0][1].ld();
    let t5 = -t4;
    let t6 = t5
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    s.quad.corners[0][1].st(t6);
    let t7 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    );
    let t8 = t7 / s.quad.corners[1][2].ld();
    let t9 = t8 * s.quad.corners[1][0].ld();
    let t10 = t9
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    s.quad.corners[1][0].st(t10);
    let t11 = t8 * s.quad.corners[1][1].ld();
    let t12 = -t11;
    let t13 = t12
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    s.quad.corners[1][1].st(t13);
    let t14 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    );
    let t15 = t14 / s.quad.corners[2][2].ld();
    let t16 = t15 * s.quad.corners[2][0].ld();
    let t17 = t16
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    s.quad.corners[2][0].st(t17);
    let t18 = t15 * s.quad.corners[2][1].ld();
    let t19 = -t18;
    let t20 = t19
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    s.quad.corners[2][1].st(t20);
    let t21 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    );
    let t22 = t21 / s.quad.corners[3][2].ld();
    let t23 = t22 * s.quad.corners[3][0].ld();
    let t24 = t23
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    s.quad.corners[3][0].st(t24);
    let t25 = t22 * s.quad.corners[3][1].ld();
    let t26 = -t25;
    let t27 = t26
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    s.quad.corners[3][1].st(t27);
}

#[cfg(test)]
mod reference_vectors {
    use super::Shadow;
    use crate::effects::asm_layout::with_laid_at;
    use crate::effects::asm_test::START;
    use crate::effects::asm_test::{Operand, floats_at, with_copy};
    use crate::effects::kind_store::{KindConsts, QuadWork};
    use crate::session::PlainFloat;

    /// A shadow's variables: where the quad, the constants and the up start in the marks' block.
    #[derive(Clone, Copy)]
    struct Layout(usize, usize, usize, u32);
    const SET_1: Layout = Layout(0x9331B4, 0x933E7C, 0x933E70, 0x26A0);
    const SET_2: Layout = Layout(0x933ED0, 0x934B7C, 0x934B78, 0x26A4);

    /// `f` on copies of a shadow's variables in the test buffer standing for the marks'
    /// block at `mk`, put back afterwards.
    fn with_shadow<R>(
        mk: crate::effects::asm_test::Operand,
        l: Layout,
        f: impl FnOnce(&Shadow) -> R,
    ) -> R {
        with_laid_at::<QuadWork, R>(mk, l.0, |quad| {
            with_consts(mk, l.1, |consts| {
                with_copy::<PlainFloat, R>(mk, l.2, |up| {
                    f(&Shadow {
                        quad,
                        consts,
                        up,
                        field: if l.3 == 0x26A0 {
                            |marks| &marks.engine_speed_copy
                        } else {
                            |marks| &marks.engine_speed_scaled_copy
                        },
                    })
                })
            })
        })
    }
    /// `f` on a set's constants made from the floats the test buffer `mk` holds at `start`.
    fn with_consts<R>(mk: Operand, start: usize, f: impl FnOnce(&KindConsts) -> R) -> R {
        let v = floats_at::<21>(mk, start);
        f(&KindConsts {
            size_k: v[0],
            start_k: v[1],
            step_k: v[2],
            middle_k: v[3],
            slope_k: v[4],
            spread_k: v[5],
            extra_k: v[6],
            place_k: [v[7], v[8], v[9], v[10], v[11], v[12]],
            place_less: v[13],
            light: [v[14], v[15], v[16]],
            lifts: [v[17], v[18], v[19], v[20]],
        })
    }
    /// `f` on the matrix and the two temporaries in the test buffers `m` and `t`.
    fn with_matrix(
        m: crate::effects::asm_test::Operand,
        t: crate::effects::asm_test::Operand,
        f: impl FnOnce(&[PlainFloat; 9], &[PlainFloat; 2]),
    ) {
        with_copy::<[PlainFloat; 9], ()>(m, START, |m| {
            with_copy::<[PlainFloat; 2], ()>(t, START, |t| f(m, t));
        });
    }
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, recorded_writes};

    #[test]
    fn light_turned_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1CEF90,
                    hi: 0x1CFC90,
                    reads: &[(0x1CFC74, F32), (0x1CFC78, F32), (0x1CFC7C, F32)],
                    writes: &[(0x1CEFA0, F32), (0x1CEFA4, F32), (0x1CEFA8, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.light_turned_x87", &SPEC, |b| {
            with_shadow(b[0], SET_2, |s| {
                with_matrix(b[1], b[2], |m, t| super::light_turned_x87(s, m, t));
            });
        });
    }

    #[test]
    fn second_turned_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1CEF90,
                    hi: 0x1CFC90,
                    reads: &[(0x1CEFA0, F32), (0x1CEFA4, F32), (0x1CEFA8, F32)],
                    writes: &[(0x1CF00C, F32), (0x1CF010, F32), (0x1CF014, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.second_turned_x87", &SPEC, |b| {
            with_shadow(b[0], SET_2, |s| {
                with_matrix(b[1], b[2], |m, t| super::second_turned_x87(s, m, t));
            });
        });
    }

    #[test]
    fn corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x1CEF90,
                hi: 0x1CFC90,
                reads: &[
                    (0x1CEF90, F32),
                    (0x1CEF94, F32),
                    (0x1CEF98, F32),
                    (0x1CEFB0, F32),
                    (0x1CEFBC, F32),
                    (0x1CEFC8, F32),
                    (0x1CF000, F32),
                    (0x1CF004, F32),
                    (0x1CF008, F32),
                    (0x1CF00C, F32),
                    (0x1CF010, F32),
                    (0x1CF014, F32),
                    (0x1CF028, F32),
                    (0x1CF02C, F32),
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF060, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF06C, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF078, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                    (0x1CF084, F32),
                    (0x1CFC38, F32),
                ],
                writes: &[
                    (0x1CF000, F32),
                    (0x1CF004, F32),
                    (0x1CF008, F32),
                    (0x1CF00C, F32),
                    (0x1CF010, F32),
                    (0x1CF014, F32),
                    (0x1CF04C, F32),
                    (0x1CF050, F32),
                    (0x1CF054, F32),
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF060, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF06C, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF078, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                    (0x1CF084, F32),
                ],
            }],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.corners_x87", &SPEC, |b| {
            with_shadow(b[0], SET_2, super::corners_x87);
        });
    }

    #[test]
    fn corners_in_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x1CEF90,
                hi: 0x1CFC90,
                reads: &[
                    (0x1CF048, F32),
                    (0x1CF04C, F32),
                    (0x1CF050, F32),
                    (0x1CF054, F32),
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF060, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF06C, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF078, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                    (0x1CF084, F32),
                ],
                writes: &[
                    (0x1CF04C, F32),
                    (0x1CF050, F32),
                    (0x1CF054, F32),
                    (0x1CF058, F32),
                    (0x1CF05C, F32),
                    (0x1CF060, F32),
                    (0x1CF064, F32),
                    (0x1CF068, F32),
                    (0x1CF06C, F32),
                    (0x1CF070, F32),
                    (0x1CF074, F32),
                    (0x1CF078, F32),
                    (0x1CF07C, F32),
                    (0x1CF080, F32),
                    (0x1CF084, F32),
                ],
            }],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.corners_in_x87", &SPEC, |b| {
            with_shadow(b[0], SET_2, super::corners_in_x87);
        });
    }

    #[test]
    fn projected_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x1CEF90,
                    hi: 0x1CFC90,
                    reads: &[
                        (0x1CF058, F32),
                        (0x1CF05C, F32),
                        (0x1CF060, F32),
                        (0x1CF064, F32),
                        (0x1CF068, F32),
                        (0x1CF06C, F32),
                        (0x1CF070, F32),
                        (0x1CF074, F32),
                        (0x1CF078, F32),
                        (0x1CF07C, F32),
                        (0x1CF080, F32),
                        (0x1CF084, F32),
                    ],
                    writes: &[
                        (0x1CF058, F32),
                        (0x1CF05C, F32),
                        (0x1CF064, F32),
                        (0x1CF068, F32),
                        (0x1CF070, F32),
                        (0x1CF074, F32),
                        (0x1CF07C, F32),
                        (0x1CF080, F32),
                    ],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.projected_x87", &SPEC, |b| {
            with_shadow(b[1], SET_2, |s| {
                super::projected_x87(s);
            });
        });
    }

    #[test]
    fn light_turned_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1CE274,
                    hi: 0x1CEF90,
                    reads: &[(0x1CEF74, F32), (0x1CEF78, F32), (0x1CEF7C, F32)],
                    writes: &[(0x1CE284, F32), (0x1CE288, F32), (0x1CE28C, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.light_turned_2_x87", &SPEC, |b| {
            with_shadow(b[0], SET_1, |s| {
                with_matrix(b[1], b[2], |m, t| super::light_turned_x87(s, m, t));
            });
        });
    }

    #[test]
    fn second_turned_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1CE274,
                    hi: 0x1CEF90,
                    reads: &[(0x1CE284, F32), (0x1CE288, F32), (0x1CE28C, F32)],
                    writes: &[(0x1CE2F0, F32), (0x1CE2F4, F32), (0x1CE2F8, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.second_turned_2_x87", &SPEC, |b| {
            with_shadow(b[0], SET_1, |s| {
                with_matrix(b[1], b[2], |m, t| super::second_turned_x87(s, m, t));
            });
        });
    }

    #[test]
    fn corners_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x1CE274,
                hi: 0x1CEF90,
                reads: &[
                    (0x1CE274, F32),
                    (0x1CE278, F32),
                    (0x1CE27C, F32),
                    (0x1CE294, F32),
                    (0x1CE2A0, F32),
                    (0x1CE2AC, F32),
                    (0x1CE2E4, F32),
                    (0x1CE2E8, F32),
                    (0x1CE2EC, F32),
                    (0x1CE2F0, F32),
                    (0x1CE2F4, F32),
                    (0x1CE2F8, F32),
                    (0x1CE30C, F32),
                    (0x1CE310, F32),
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE344, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE350, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE35C, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                    (0x1CE368, F32),
                    (0x1CEF30, F32),
                ],
                writes: &[
                    (0x1CE2E4, F32),
                    (0x1CE2E8, F32),
                    (0x1CE2EC, F32),
                    (0x1CE2F0, F32),
                    (0x1CE2F4, F32),
                    (0x1CE2F8, F32),
                    (0x1CE330, F32),
                    (0x1CE334, F32),
                    (0x1CE338, F32),
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE344, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE350, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE35C, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                    (0x1CE368, F32),
                ],
            }],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.corners_2_x87", &SPEC, |b| {
            with_shadow(b[0], SET_1, super::corners_x87);
        });
    }

    #[test]
    fn corners_in_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x1CE274,
                hi: 0x1CEF90,
                reads: &[
                    (0x1CE32C, F32),
                    (0x1CE330, F32),
                    (0x1CE334, F32),
                    (0x1CE338, F32),
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE344, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE350, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE35C, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                    (0x1CE368, F32),
                ],
                writes: &[
                    (0x1CE330, F32),
                    (0x1CE334, F32),
                    (0x1CE338, F32),
                    (0x1CE33C, F32),
                    (0x1CE340, F32),
                    (0x1CE344, F32),
                    (0x1CE348, F32),
                    (0x1CE34C, F32),
                    (0x1CE350, F32),
                    (0x1CE354, F32),
                    (0x1CE358, F32),
                    (0x1CE35C, F32),
                    (0x1CE360, F32),
                    (0x1CE364, F32),
                    (0x1CE368, F32),
                ],
            }],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.corners_in_2_x87", &SPEC, |b| {
            with_shadow(b[0], SET_1, super::corners_in_x87);
        });
    }

    #[test]
    fn projected_2_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x1CE274,
                    hi: 0x1CEF90,
                    reads: &[
                        (0x1CE33C, F32),
                        (0x1CE340, F32),
                        (0x1CE344, F32),
                        (0x1CE348, F32),
                        (0x1CE34C, F32),
                        (0x1CE350, F32),
                        (0x1CE354, F32),
                        (0x1CE358, F32),
                        (0x1CE35C, F32),
                        (0x1CE360, F32),
                        (0x1CE364, F32),
                        (0x1CE368, F32),
                    ],
                    writes: &[
                        (0x1CE33C, F32),
                        (0x1CE340, F32),
                        (0x1CE348, F32),
                        (0x1CE34C, F32),
                        (0x1CE354, F32),
                        (0x1CE358, F32),
                        (0x1CE360, F32),
                        (0x1CE364, F32),
                    ],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_shadow.projected_2_x87", &SPEC, |b| {
            with_shadow(b[1], SET_1, |s| {
                super::projected_x87(s);
            });
        });
    }
}
