//! `effects::draw_shadow`: the car's ground shadow (probably: the quad is drawn flat on the ground
//! under the car), unless the quad's `corner` is -1. The depth buffer and test are switched on
//! (`scene::depth_buffer_on`, `scene::depth_test_on`) with render states 0x15 = 4, 0x13 = 5, 0x14 =
//! 6, 0x1C = 0, 0x1B = 1. The quad's two `sides` are made one long and square
//! (`scene::cross_product3`, `scene::normalise_vector3`); its `matrix` is turned about z by an
//! angle from the settings' engine speed copy (two lines, by the quad's `ratio`) and the light's
//! way turned by it; the quad's `corners` about its `place` (the `frame`) are drawn in a little and
//! turned to the view (`scene::rotate_point`); the texture places come from the mark corner the
//! quad's `corner` names and the colour is the shadow colour | 0xFF000000 brought halfway to white.
//! Unless a corner's depth is at or below the nearest depth (signed compares of the bits) the
//! corners are projected, their depths less the quad's lift times a depth scale, and drawn with
//! `scene::draw_fan`.
//!
//! The long x87 runs are plain Rust on the x87 model (`math::x87_float`); the vector helpers are
//! Rust too. Refuses, before anything, a corner past the mark corners table and an angle with no
//! sine and cosine.

use crate::effects::kind_store::{CONSTS_1, CONSTS_2, KindConsts, QuadWork};
use crate::effects::mask_store::Masks;
use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{Axis, extended, rounded_value, turn_matrix3_plain, turnable};
use crate::math::x87;
use crate::memory::session::PlainFloat;
use crate::scene::cross_product3::cross_plain;
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::normalise_plain;

/// A shadow's variables (the typed statics of `effects::kind_store`: the second shadow's,
/// `effects::draw_overlays`'s, are set 1's, the first's set 2's) and its x87 blocks.
pub struct Shadow<'a> {
    pub quad: &'a core::cell::RefCell<QuadWork>,
    pub consts: &'a KindConsts,
    /// The quad is lifted along it.
    pub up: &'a core::cell::Cell<f32>,
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
        let q = sh.quad.borrow();
        let split = q.low;
        let v = (sh.field)(marks).get();
        if x87::bits_at_or_below(split, v) {
            x87::less_times_plus_negated_80(v, split, q.slope, q.middle)
        } else {
            x87::times_plus_negated_80(v, q.step, q.start)
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
        self.quad.borrow().corner
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
pub(super) fn draw_shadow_with(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    masks: &Masks,
    sh: &Shadow,
    depth: bool,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let buffer = &ctx.part.vertices;
    let race = ctx.race;
    use crate::scene::set_render_state::set_render_state;
    let corner: u32 = sh.quad.borrow().corner;
    if corner == 0xFFFF_FFFF {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a shadow corner past g_marksCorners").into());
    }
    effects.settings()?;
    turnable(&angle(sh, effects)?)?;
    let mut temps = [0.0f32; 2];

    if depth {
        crate::scene::depth_buffer_on::depth_buffer(ctx.part, true);
        crate::scene::depth_test_on::depth_test_on(ctx.part);
    }
    for (value, state) in STATES {
        set_render_state(ctx.part, value, state);
    }
    {
        let mut q = sh.quad.borrow_mut();
        let q = &mut *q;
        for n in 0..2 {
            q.frame[n] = q.sides[n];
        }
        normalise_plain(&mut q.frame[0]);
        normalise_plain(&mut q.frame[1]);
        q.frame[2] = cross_plain(&q.frame[0], &q.frame[1]);
        q.frame[1] = cross_plain(&q.frame[2], &q.frame[0]);
        normalise_plain(&mut q.frame[1]);
        normalise_plain(&mut q.frame[2]);
    }
    let a = angle(sh, effects)?;
    {
        let mut q = sh.quad.borrow_mut();
        turn_matrix3_plain(Axis::Z, &mut q.matrix, a)?;
    }
    let matrix = sh.quad.borrow().matrix;
    light_turned_x87(sh, &matrix, &mut temps);
    {
        let mut q = sh.quad.borrow_mut();
        let q = &mut *q;
        for n in 0..3 {
            for k in 0..3 {
                q.matrix[n + 3 * k] = q.frame[n][k];
            }
        }
    }
    let matrix = sh.quad.borrow().matrix;
    second_turned_x87(sh, &matrix, &mut temps);
    {
        let mut q = sh.quad.borrow_mut();
        let q = &mut *q;
        q.frame[0] = cross_plain(&q.frame[2], &q.frame[1]);
        q.frame[1] = cross_plain(&q.frame[2], &q.frame[0]);
    }
    corners_x87(sh);
    {
        let mut q = sh.quad.borrow_mut();
        normalise_plain(&mut q.way);
    }
    corners_in_x87(sh);
    let view = &race.frame.matrices.get().listener_matrix;
    {
        let mut q = sh.quad.borrow_mut();
        for item in q.corners.iter_mut().take(4) {
            let turned = rotate_point(view, (*item).map(extended));
            for (k, v) in turned.into_iter().enumerate() {
                item[k] = rounded_value(v);
            }
        }
    }
    let entry = &effects.blocks.corners.corners[corner as usize]; // under 0x100, checked before
    let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
    let uv_k = effects.frame.draw_work.borrow().uv_k;
    for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
        let vertex = &buffer.vertices[n];
        vertex.u.set(x87::int_times(word(u) as i32, uv_k));
        vertex.v.set(x87::int_times(word(v) as i32, uv_k));
    }
    let mut c = masks.surface.borrow().lives_by_level[8] | 0xFF00_0000;
    let half = |shift: u32| ((!(c >> shift) & 0xFF) >> 1) << shift;
    let (r, gr, bl) = (half(16), half(8), half(0));
    c = c.wrapping_add(r).wrapping_add(gr).wrapping_add(bl);
    for item in buffer.vertices.iter().take(4) {
        item.colour.set(c);
    }
    let q = sh.quad.borrow();
    for item in q.corners.iter().take(4) {
        if x87::bits_at_or_below(item[2], race.course.numbers.k4096_a.get()) {
            return Ok(());
        }
    }
    drop(q);
    projected_x87(ctx.scene.scratch.front_end_polygon(), sh);
    let q = sh.quad.borrow();
    let z_k = effects.frame.draw_work.borrow().z_k;
    for n in 0..4 {
        let vertex = &buffer.vertices[n];
        vertex.x.set(q.corners[n][0]);
        vertex.y.set(q.corners[n][1]);
        let depth = x87::sub_mul(q.corners[n][2], sh.consts.lifts[n], z_k);
        vertex.z.set(depth);
    }
    crate::effects::clip_quad::draw_fan(buffer, ctx.scene);
    Ok(())
}

use crate::math::x87_float as bh;
use crate::race::effects_settings_store::EffectsSettings;

/// `light_turned`'s arithmetic as plain Rust on the x87 model, in the same order, on either
/// shadow's variables.
fn light_turned_x87(s: &Shadow, m: &[f32; 9], t: &mut [f32; 2]) {
    let mut q = s.quad.borrow_mut();
    let t0 = bh::ld(s.consts.light[0]);
    let t1 = bh::ld(s.consts.light[1]);
    let t2 = bh::ld(s.consts.light[2]);
    let t3 = t2 * bh::ld(m[0]);
    let t4 = t1 * bh::ld(m[1]);
    let t5 = t0 * bh::ld(m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    t[0] = bh::stf(t7);
    let t8 = t2 * bh::ld(m[3]);
    let t9 = t1 * bh::ld(m[4]);
    let t10 = t0 * bh::ld(m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    t[1] = bh::stf(t12);
    let t13 = t2 * bh::ld(m[6]);
    let t14 = t1 * bh::ld(m[7]);
    let t15 = t0 * bh::ld(m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = bh::ld(t[1]);
    let t19 = bh::ld(t[0]);
    q.light_turned[0] = bh::stf(t19);
    q.light_turned[1] = bh::stf(t18);
    q.light_turned[2] = bh::stf(t17);
}

/// The second light turn's arithmetic as plain Rust on the x87 model, in the same order, on
/// either shadow's variables.
fn second_turned_x87(s: &Shadow, m: &[f32; 9], t: &mut [f32; 2]) {
    let mut q = s.quad.borrow_mut();
    let t0 = bh::ld(q.light_turned[2]);
    let t1 = bh::ld(q.light_turned[1]);
    let t2 = bh::ld(q.light_turned[0]);
    let t3 = t2 * bh::ld(m[0]);
    let t4 = t1 * bh::ld(m[1]);
    let t5 = t0 * bh::ld(m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    t[0] = bh::stf(t7);
    let t8 = t2 * bh::ld(m[3]);
    let t9 = t1 * bh::ld(m[4]);
    let t10 = t0 * bh::ld(m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    t[1] = bh::stf(t12);
    let t13 = t2 * bh::ld(m[6]);
    let t14 = t1 * bh::ld(m[7]);
    let t15 = t0 * bh::ld(m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = bh::ld(t[1]);
    let t19 = bh::ld(t[0]);
    q.frame[1][0] = bh::stf(t19);
    q.frame[1][1] = bh::stf(t18);
    q.frame[1][2] = bh::stf(t17);
}

/// `corners`'s arithmetic as plain Rust on the x87 model, in the same order, on either shadow's
/// variables.
fn corners_x87(s: &Shadow) {
    let up = bh::ld(s.up.get());
    let mut q = s.quad.borrow_mut();
    let t0 = bh::ld(q.frame[0][0]);
    let t1 = t0 * bh::ld(q.ratio);
    q.frame[0][0] = bh::stf(t1);
    let t2 = bh::ld(q.frame[0][1]);
    let t3 = t2 * bh::ld(q.ratio);
    q.frame[0][1] = bh::stf(t3);
    let t4 = bh::ld(q.frame[0][2]);
    let t5 = t4 * bh::ld(q.ratio);
    q.frame[0][2] = bh::stf(t5);
    let t6 = bh::ld(q.frame[1][0]);
    let t7 = t6 * bh::ld(q.size);
    q.frame[1][0] = bh::stf(t7);
    let t8 = bh::ld(q.frame[1][1]);
    let t9 = t8 * bh::ld(q.size);
    q.frame[1][1] = bh::stf(t9);
    let t10 = bh::ld(q.frame[1][2]);
    let t11 = t10 * bh::ld(q.size);
    q.frame[1][2] = bh::stf(t11);
    let t12 = bh::ld(q.place[0]);
    let t13 = t12 - bh::ld(q.frame[0][0]);
    q.corners[0][0] = bh::stf(t13);
    let t14 = bh::ld(q.place[1]);
    let t15 = t14 - bh::ld(q.frame[0][1]);
    q.corners[0][1] = bh::stf(t15);
    let t16 = bh::ld(q.place[2]);
    let t17 = t16 - bh::ld(q.frame[0][2]);
    q.corners[0][2] = bh::stf(t17);
    let t18 = bh::ld(q.place[0]);
    let t19 = t18 + bh::ld(q.frame[0][0]);
    q.corners[1][0] = bh::stf(t19);
    let t20 = bh::ld(q.place[1]);
    let t21 = t20 + bh::ld(q.frame[0][1]);
    q.corners[1][1] = bh::stf(t21);
    let t22 = bh::ld(q.place[2]);
    let t23 = t22 + bh::ld(q.frame[0][2]);
    q.corners[1][2] = bh::stf(t23);
    let t24 = bh::ld(q.place[0]);
    let t25 = t24 + bh::ld(q.frame[0][0]);
    let t26 = t25 + bh::ld(q.frame[1][0]);
    q.corners[2][0] = bh::stf(t26);
    let t27 = bh::ld(q.place[1]);
    let t28 = t27 + bh::ld(q.frame[0][1]);
    let t29 = t28 + bh::ld(q.frame[1][1]);
    q.corners[2][1] = bh::stf(t29);
    let t30 = bh::ld(q.place[2]);
    let t31 = t30 + bh::ld(q.frame[0][2]);
    let t32 = t31 + bh::ld(q.frame[1][2]);
    q.corners[2][2] = bh::stf(t32);
    let t33 = bh::ld(q.place[0]);
    let t34 = t33 - bh::ld(q.frame[0][0]);
    let t35 = t34 + bh::ld(q.frame[1][0]);
    q.corners[3][0] = bh::stf(t35);
    let t36 = bh::ld(q.place[1]);
    let t37 = t36 - bh::ld(q.frame[0][1]);
    let t38 = t37 + bh::ld(q.frame[1][1]);
    q.corners[3][1] = bh::stf(t38);
    let t39 = bh::ld(q.place[2]);
    let t40 = t39 - bh::ld(q.frame[0][2]);
    let t41 = t40 + bh::ld(q.frame[1][2]);
    q.corners[3][2] = bh::stf(t41);
    let t42 = bh::ld(q.matrix[1]);
    let t43 = t42 * up;
    let t44 = t43 + bh::ld(q.corners[0][0]);
    q.corners[0][0] = bh::stf(t44);
    let t45 = bh::ld(q.matrix[4]);
    let t46 = t45 * up;
    let t47 = t46 + bh::ld(q.corners[0][1]);
    q.corners[0][1] = bh::stf(t47);
    let t48 = bh::ld(q.matrix[7]);
    let t49 = t48 * up;
    let t50 = t49 + bh::ld(q.corners[0][2]);
    q.corners[0][2] = bh::stf(t50);
    let t51 = bh::ld(q.matrix[1]);
    let t52 = t51 * up;
    let t53 = t52 + bh::ld(q.corners[1][0]);
    q.corners[1][0] = bh::stf(t53);
    let t54 = bh::ld(q.matrix[4]);
    let t55 = t54 * up;
    let t56 = t55 + bh::ld(q.corners[1][1]);
    q.corners[1][1] = bh::stf(t56);
    let t57 = bh::ld(q.matrix[7]);
    let t58 = t57 * up;
    let t59 = t58 + bh::ld(q.corners[1][2]);
    q.corners[1][2] = bh::stf(t59);
    let t60 = bh::ld(q.matrix[1]);
    let t61 = t60 * up;
    let t62 = t61 + bh::ld(q.corners[2][0]);
    q.corners[2][0] = bh::stf(t62);
    let t63 = bh::ld(q.matrix[4]);
    let t64 = t63 * up;
    let t65 = t64 + bh::ld(q.corners[2][1]);
    q.corners[2][1] = bh::stf(t65);
    let t66 = bh::ld(q.matrix[7]);
    let t67 = t66 * up;
    let t68 = t67 + bh::ld(q.corners[2][2]);
    q.corners[2][2] = bh::stf(t68);
    let t69 = bh::ld(q.matrix[1]);
    let t70 = t69 * up;
    let t71 = t70 + bh::ld(q.corners[3][0]);
    q.corners[3][0] = bh::stf(t71);
    let t72 = bh::ld(q.matrix[4]);
    let t73 = t72 * up;
    let t74 = t73 + bh::ld(q.corners[3][1]);
    q.corners[3][1] = bh::stf(t74);
    let t75 = bh::ld(q.matrix[7]);
    let t76 = t75 * up;
    let t77 = t76 + bh::ld(q.corners[3][2]);
    q.corners[3][2] = bh::stf(t77);
    let t78 = bh::ld(q.corners[2][0]);
    let t79 = t78 - bh::ld(q.corners[1][0]);
    q.way[0] = bh::stf(t79);
    let t80 = bh::ld(q.corners[2][1]);
    let t81 = t80 - bh::ld(q.corners[1][1]);
    q.way[1] = bh::stf(t81);
    let t82 = bh::ld(q.corners[2][2]);
    let t83 = t82 - bh::ld(q.corners[1][2]);
    q.way[2] = bh::stf(t83);
}

/// The arithmetic on the corners, after the way is normalised, as plain Rust on the x87 model, in
/// the same order, on either shadow's variables.
fn corners_in_x87(s: &Shadow) {
    let mut q = s.quad.borrow_mut();
    let t0 = bh::ld(q.way[0]);
    let t1 = t0 * bh::ld(q.spread);
    q.way[0] = bh::stf(t1);
    let t2 = bh::ld(q.way[1]);
    let t3 = t2 * bh::ld(q.spread);
    q.way[1] = bh::stf(t3);
    let t4 = bh::ld(q.way[2]);
    let t5 = t4 * bh::ld(q.spread);
    q.way[2] = bh::stf(t5);
    let t6 = bh::ld(q.corners[0][0]);
    let t7 = t6 - bh::ld(q.way[0]);
    q.corners[0][0] = bh::stf(t7);
    let t8 = bh::ld(q.corners[0][1]);
    let t9 = t8 - bh::ld(q.way[1]);
    q.corners[0][1] = bh::stf(t9);
    let t10 = bh::ld(q.corners[0][2]);
    let t11 = t10 - bh::ld(q.way[2]);
    q.corners[0][2] = bh::stf(t11);
    let t12 = bh::ld(q.corners[1][0]);
    let t13 = t12 - bh::ld(q.way[0]);
    q.corners[1][0] = bh::stf(t13);
    let t14 = bh::ld(q.corners[1][1]);
    let t15 = t14 - bh::ld(q.way[1]);
    q.corners[1][1] = bh::stf(t15);
    let t16 = bh::ld(q.corners[1][2]);
    let t17 = t16 - bh::ld(q.way[2]);
    q.corners[1][2] = bh::stf(t17);
    let t18 = bh::ld(q.corners[2][0]);
    let t19 = t18 - bh::ld(q.way[0]);
    q.corners[2][0] = bh::stf(t19);
    let t20 = bh::ld(q.corners[2][1]);
    let t21 = t20 - bh::ld(q.way[1]);
    q.corners[2][1] = bh::stf(t21);
    let t22 = bh::ld(q.corners[2][2]);
    let t23 = t22 - bh::ld(q.way[2]);
    q.corners[2][2] = bh::stf(t23);
    let t24 = bh::ld(q.corners[3][0]);
    let t25 = t24 - bh::ld(q.way[0]);
    q.corners[3][0] = bh::stf(t25);
    let t26 = bh::ld(q.corners[3][1]);
    let t27 = t26 - bh::ld(q.way[1]);
    q.corners[3][1] = bh::stf(t27);
    let t28 = bh::ld(q.corners[3][2]);
    let t29 = t28 - bh::ld(q.way[2]);
    q.corners[3][2] = bh::stf(t29);
}

/// `projected`'s arithmetic as plain Rust on the x87 model, in the same order, on either shadow's
/// variables.
fn projected_x87(front: &crate::scene::polygon_work_store::PolygonWork, s: &Shadow) {
    let mut q = s.quad.borrow_mut();
    let t0 = crate::math::x87_float::ld(front.window.borrow().view_scale);
    let t1 = t0 / bh::ld(q.corners[0][2]);
    let t2 = t1 * bh::ld(q.corners[0][0]);
    let t3 = t2 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    q.corners[0][0] = bh::stf(t3);
    let t4 = t1 * bh::ld(q.corners[0][1]);
    let t5 = -t4;
    let t6 = t5 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    q.corners[0][1] = bh::stf(t6);
    let t7 = crate::math::x87_float::ld(front.window.borrow().view_scale);
    let t8 = t7 / bh::ld(q.corners[1][2]);
    let t9 = t8 * bh::ld(q.corners[1][0]);
    let t10 = t9 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    q.corners[1][0] = bh::stf(t10);
    let t11 = t8 * bh::ld(q.corners[1][1]);
    let t12 = -t11;
    let t13 = t12 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    q.corners[1][1] = bh::stf(t13);
    let t14 = crate::math::x87_float::ld(front.window.borrow().view_scale);
    let t15 = t14 / bh::ld(q.corners[2][2]);
    let t16 = t15 * bh::ld(q.corners[2][0]);
    let t17 = t16 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    q.corners[2][0] = bh::stf(t17);
    let t18 = t15 * bh::ld(q.corners[2][1]);
    let t19 = -t18;
    let t20 = t19 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    q.corners[2][1] = bh::stf(t20);
    let t21 = crate::math::x87_float::ld(front.window.borrow().view_scale);
    let t22 = t21 / bh::ld(q.corners[3][2]);
    let t23 = t22 * bh::ld(q.corners[3][0]);
    let t24 = t23 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    q.corners[3][0] = bh::stf(t24);
    let t25 = t22 * bh::ld(q.corners[3][1]);
    let t26 = -t25;
    let t27 = t26 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    q.corners[3][1] = bh::stf(t27);
}

#[cfg(test)]
mod tests;
