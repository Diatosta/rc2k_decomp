//! `effects::draw_shadow` (0x4C8168): the car's ground shadow (a reading), unless
//! `dword_933F64` (its corner) is -1: `scene::depth_buffer_on`, `scene::depth_test_on`,
//! render states 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1; two sides
//! (`dword_933F1C` .., `dword_933F28` ..) made one long and square (`scene::cross_product3`,
//! `scene::normalise_vector3`); the matrix at `dword_933EEC` turned about z by an angle
//! from the settings' +0x26A4 (two lines, by `dword_933F78`); the light's way turned by it,
//! the sides made its columns, the way turned again and the sides crossed once more; the
//! quad's corners about `flt_933ED0` (`flt_933F98` ..), drawn in a little, turned to the
//! view (`unk_5EA0CC`, `scene::rotate_point`); texture places from corner `dword_933F64`
//! of `g_marksCorners`; the colour `dword_939BA6` | 0xFF000000 brought halfway to white;
//! unless a corner's depth is at or below `flt_5C5E18` (signed compares of the bits): the
//! corners projected, their depths less `flt_934BC0` .. times `flt_8EB96D`, and
//! `scene::draw_fan`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: the race's frame (0x456330). The long x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order,
//! over the game's variables; the vector helpers their Rust versions. Refuses, before
//! anything, a corner past `g_marksCorners` and an angle `fsincos` cannot take.

use crate::effects::blocks::MARK_CORNERS;
use crate::effects::frame_state::DRAW_WORK;
use crate::effects::kind_store::{
    CONSTS_1, CONSTS_2, EXTRA_2, EXTRAS_1, KindConsts, QUAD_1, QUAD_2, QuadWork,
};
use crate::effects::mask_store::Masks;
use crate::errors::{BadInput, Refused};
use crate::math::{x87, x87_run};
use crate::race::owner::Race;
use crate::scene::cross_product3::cross;
use crate::scene::matrix3::{Axis, extended, rounded, turn_matrix3, turnable};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::normalise;
use crate::session::Float;

use crate::d3d::vertices_store::VERTICES;

/// A shadow's variables (the typed statics of `effects::kind_store`: the second shadow's,
/// `sub_4C74DA`'s, are set 1's, the first's set 2's) and its x87 blocks.
pub struct Shadow<'a> {
    pub quad: &'a QuadWork,
    pub consts: &'a KindConsts,
    /// The quad is lifted along it (`flt_934B78`, set 1's `flt_933E70`).
    pub up: &'a Float,
    /// The settings' field the angle comes from.
    pub field: fn(&MarksSettings) -> &Float,
}

/// `draw_shadow`'s.
pub static FIRST: Shadow<'static> = Shadow {
    quad: &QUAD_2,
    consts: &CONSTS_2,
    up: &EXTRA_2.extra,
    field: |marks| &marks.listener_float_b,
};

/// `sub_4C74DA`'s.
pub static SECOND: Shadow<'static> = Shadow {
    quad: &QUAD_1,
    consts: &CONSTS_1,
    up: &EXTRAS_1.extra,
    field: |marks| &marks.listener_float_a,
};

/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The z turn's angle (see the header), 80 bits.
fn angle(sh: &Shadow) -> Result<[u8; 10], Refused> {
    Ok({
        let marks = crate::effects::setup::marks()?;
        let split: u32 = sh.quad.low.bits();
        let v = (sh.field)(marks).bits();
        if (split as i32) <= v as i32 {
            x87::less_times_plus_negated_80(v, split, sh.quad.slope.bits(), sh.quad.middle.bits())
        } else {
            x87::times_plus_negated_80(v, sh.quad.step.bits(), sh.quad.start.bits())
        }
    })
}

/// The Rust version (see the header).
pub fn draw_shadow(race: &Race, masks: &Masks) -> Result<(), Refused> {
    draw_shadow_with(race, masks, &FIRST, true)
}

impl Shadow<'_> {
    /// The corner (-1: no shadow).
    pub fn corner(&self) -> u32 {
        self.quad.corner.get()
    }

    /// The angle's refusal, if any (for a caller that checks before drawing).
    pub fn angle_refuses(&self) -> Option<Refused> {
        match angle(self) {
            Err(e) => Some(e),
            Ok(a) => { turnable(&a) }.err(),
        }
    }
}

/// A shadow (see the header) on `sh`'s variables; `depth` the two depth calls first.
pub fn draw_shadow_with(
    race: &Race,
    masks: &Masks,
    sh: &Shadow,
    depth: bool,
) -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    let q = sh.quad;
    let corner: u32 = q.corner.get();
    if corner == 0xFFFF_FFFF {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a shadow corner past g_marksCorners").into());
    }
    crate::effects::setup::marks()?;
    turnable(&angle(sh)?)?;
    let temps = [const { Float::from_bits(0) }; 2];

    if depth {
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::depth_test_on::depth_test_on();
    }
    for (value, state) in STATES {
        set_render_state(value, state);
    }
    for n in 0..2 {
        for k in 0..3 {
            q.frame[n][k].set_bits(q.sides[n][k].bits());
        }
    }
    let [s0, s1, s2] = [0, 1, 2].map(|n| &q.frame[n]);
    normalise(s0);
    normalise(s1);
    cross(s0, s1, s2);
    cross(s2, s0, s1);
    normalise(s1);
    normalise(s2);
    let a = angle(sh)?;
    turn_matrix3(Axis::Z, &q.matrix, a)?;
    light_turned_x87(sh, &q.matrix, &temps);
    for n in 0..3 {
        for k in 0..3 {
            q.matrix[n + 3 * k].set_bits(q.frame[n][k].bits());
        }
    }
    second_turned_x87(sh, &q.matrix, &temps);
    cross(s2, s1, s0);
    cross(s2, s0, s1);
    corners_x87(sh);
    normalise(&q.way);
    corners_in_x87(sh);
    let view = &race.frame.matrices.listener_matrix;
    for n in 0..4 {
        let p: [u32; 3] = std::array::from_fn(|k| q.corners[n][k].bits());
        let turned = rotate_point(view, p.map(extended));
        for (k, v) in turned.into_iter().enumerate() {
            q.corners[n][k].set_bits(rounded(v));
        }
    }
    let entry = &MARK_CORNERS.corners[corner as usize]; // under 0x100, checked before
    let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
    let uv_k: u32 = DRAW_WORK.uv_k.bits();
    for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
        let vertex = &VERTICES.vertices[n];
        vertex.u.set_bits(x87::int_times(word(u) as i32, uv_k));
        vertex.v.set_bits(x87::int_times(word(v) as i32, uv_k));
    }
    let mut c = masks.surface.lives_by_level[8].get() | 0xFF00_0000;
    let half = |shift: u32| ((!(c >> shift) & 0xFF) >> 1) << shift;
    let (r, gr, bl) = (half(16), half(8), half(0));
    c = c.wrapping_add(r).wrapping_add(gr).wrapping_add(bl);
    for n in 0..4 {
        VERTICES.vertices[n].colour.set(c);
    }
    for n in 0..4 {
        if (q.corners[n][2].bits() as i32) <= (race.course.numbers.k4096_a.bits() as i32) {
            return Ok(());
        }
    }
    projected_x87(sh);
    let z_k: u32 = DRAW_WORK.z_k.bits();
    for n in 0..4 {
        let vertex = &VERTICES.vertices[n];
        vertex.x.set_bits(q.corners[n][0].bits());
        vertex.y.set_bits(q.corners[n][1].bits());
        let depth = x87_run::sub_mul(q.corners[n][2].bits(), sh.consts.lifts[n].to_bits(), z_k);
        vertex.z.set_bits(depth);
    }
    crate::effects::clip_quad::draw_fan();
    Ok(())
}

use crate::effects::plain::{ld_cell, st_cell};
use crate::math::x87_float as bh;
use crate::race::marks_settings_store::MarksSettings;
use crate::scene::draw_store::DRAW;

/// `light_turned`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places, on either shadow's variables.
fn light_turned_x87(s: &Shadow, m: &[Float; 9], t: &[Float; 2]) {
    let t0 = bh::ld(s.consts.light[0].to_bits());
    let t1 = bh::ld(s.consts.light[1].to_bits());
    let t2 = bh::ld(s.consts.light[2].to_bits());
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
    s.quad.light_turned[0].set_bits(bh::st(t19));
    s.quad.light_turned[1].set_bits(bh::st(t18));
    s.quad.light_turned[2].set_bits(bh::st(t17));
}

/// `second_turned`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places, on either shadow's variables.
fn second_turned_x87(s: &Shadow, m: &[Float; 9], t: &[Float; 2]) {
    let t0 = bh::ld(s.quad.light_turned[2].bits());
    let t1 = bh::ld(s.quad.light_turned[1].bits());
    let t2 = bh::ld(s.quad.light_turned[0].bits());
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
    s.quad.frame[1][0].set_bits(bh::st(t19));
    s.quad.frame[1][1].set_bits(bh::st(t18));
    s.quad.frame[1][2].set_bits(bh::st(t17));
}

/// `corners`'s x87 block as plain Rust on the x87 model (117 instructions), same order and memory places, on either shadow's variables.
fn corners_x87(s: &Shadow) {
    let t0 = bh::ld(s.quad.frame[0][0].bits());
    let t1 = t0 * bh::ld(s.quad.ratio.bits());
    s.quad.frame[0][0].set_bits(bh::st(t1));
    let t2 = bh::ld(s.quad.frame[0][1].bits());
    let t3 = t2 * bh::ld(s.quad.ratio.bits());
    s.quad.frame[0][1].set_bits(bh::st(t3));
    let t4 = bh::ld(s.quad.frame[0][2].bits());
    let t5 = t4 * bh::ld(s.quad.ratio.bits());
    s.quad.frame[0][2].set_bits(bh::st(t5));
    let t6 = bh::ld(s.quad.frame[1][0].bits());
    let t7 = t6 * bh::ld(s.quad.size.bits());
    s.quad.frame[1][0].set_bits(bh::st(t7));
    let t8 = bh::ld(s.quad.frame[1][1].bits());
    let t9 = t8 * bh::ld(s.quad.size.bits());
    s.quad.frame[1][1].set_bits(bh::st(t9));
    let t10 = bh::ld(s.quad.frame[1][2].bits());
    let t11 = t10 * bh::ld(s.quad.size.bits());
    s.quad.frame[1][2].set_bits(bh::st(t11));
    let t12 = bh::ld(s.quad.place[0].bits());
    let t13 = t12 - bh::ld(s.quad.frame[0][0].bits());
    s.quad.corners[0][0].set_bits(bh::st(t13));
    let t14 = bh::ld(s.quad.place[1].bits());
    let t15 = t14 - bh::ld(s.quad.frame[0][1].bits());
    s.quad.corners[0][1].set_bits(bh::st(t15));
    let t16 = bh::ld(s.quad.place[2].bits());
    let t17 = t16 - bh::ld(s.quad.frame[0][2].bits());
    s.quad.corners[0][2].set_bits(bh::st(t17));
    let t18 = bh::ld(s.quad.place[0].bits());
    let t19 = t18 + bh::ld(s.quad.frame[0][0].bits());
    s.quad.corners[1][0].set_bits(bh::st(t19));
    let t20 = bh::ld(s.quad.place[1].bits());
    let t21 = t20 + bh::ld(s.quad.frame[0][1].bits());
    s.quad.corners[1][1].set_bits(bh::st(t21));
    let t22 = bh::ld(s.quad.place[2].bits());
    let t23 = t22 + bh::ld(s.quad.frame[0][2].bits());
    s.quad.corners[1][2].set_bits(bh::st(t23));
    let t24 = bh::ld(s.quad.place[0].bits());
    let t25 = t24 + bh::ld(s.quad.frame[0][0].bits());
    let t26 = t25 + bh::ld(s.quad.frame[1][0].bits());
    s.quad.corners[2][0].set_bits(bh::st(t26));
    let t27 = bh::ld(s.quad.place[1].bits());
    let t28 = t27 + bh::ld(s.quad.frame[0][1].bits());
    let t29 = t28 + bh::ld(s.quad.frame[1][1].bits());
    s.quad.corners[2][1].set_bits(bh::st(t29));
    let t30 = bh::ld(s.quad.place[2].bits());
    let t31 = t30 + bh::ld(s.quad.frame[0][2].bits());
    let t32 = t31 + bh::ld(s.quad.frame[1][2].bits());
    s.quad.corners[2][2].set_bits(bh::st(t32));
    let t33 = bh::ld(s.quad.place[0].bits());
    let t34 = t33 - bh::ld(s.quad.frame[0][0].bits());
    let t35 = t34 + bh::ld(s.quad.frame[1][0].bits());
    s.quad.corners[3][0].set_bits(bh::st(t35));
    let t36 = bh::ld(s.quad.place[1].bits());
    let t37 = t36 - bh::ld(s.quad.frame[0][1].bits());
    let t38 = t37 + bh::ld(s.quad.frame[1][1].bits());
    s.quad.corners[3][1].set_bits(bh::st(t38));
    let t39 = bh::ld(s.quad.place[2].bits());
    let t40 = t39 - bh::ld(s.quad.frame[0][2].bits());
    let t41 = t40 + bh::ld(s.quad.frame[1][2].bits());
    s.quad.corners[3][2].set_bits(bh::st(t41));
    let t42 = bh::ld(s.quad.matrix[1].bits());
    let t43 = t42 * bh::ld(s.up.bits());
    let t44 = t43 + bh::ld(s.quad.corners[0][0].bits());
    s.quad.corners[0][0].set_bits(bh::st(t44));
    let t45 = bh::ld(s.quad.matrix[4].bits());
    let t46 = t45 * bh::ld(s.up.bits());
    let t47 = t46 + bh::ld(s.quad.corners[0][1].bits());
    s.quad.corners[0][1].set_bits(bh::st(t47));
    let t48 = bh::ld(s.quad.matrix[7].bits());
    let t49 = t48 * bh::ld(s.up.bits());
    let t50 = t49 + bh::ld(s.quad.corners[0][2].bits());
    s.quad.corners[0][2].set_bits(bh::st(t50));
    let t51 = bh::ld(s.quad.matrix[1].bits());
    let t52 = t51 * bh::ld(s.up.bits());
    let t53 = t52 + bh::ld(s.quad.corners[1][0].bits());
    s.quad.corners[1][0].set_bits(bh::st(t53));
    let t54 = bh::ld(s.quad.matrix[4].bits());
    let t55 = t54 * bh::ld(s.up.bits());
    let t56 = t55 + bh::ld(s.quad.corners[1][1].bits());
    s.quad.corners[1][1].set_bits(bh::st(t56));
    let t57 = bh::ld(s.quad.matrix[7].bits());
    let t58 = t57 * bh::ld(s.up.bits());
    let t59 = t58 + bh::ld(s.quad.corners[1][2].bits());
    s.quad.corners[1][2].set_bits(bh::st(t59));
    let t60 = bh::ld(s.quad.matrix[1].bits());
    let t61 = t60 * bh::ld(s.up.bits());
    let t62 = t61 + bh::ld(s.quad.corners[2][0].bits());
    s.quad.corners[2][0].set_bits(bh::st(t62));
    let t63 = bh::ld(s.quad.matrix[4].bits());
    let t64 = t63 * bh::ld(s.up.bits());
    let t65 = t64 + bh::ld(s.quad.corners[2][1].bits());
    s.quad.corners[2][1].set_bits(bh::st(t65));
    let t66 = bh::ld(s.quad.matrix[7].bits());
    let t67 = t66 * bh::ld(s.up.bits());
    let t68 = t67 + bh::ld(s.quad.corners[2][2].bits());
    s.quad.corners[2][2].set_bits(bh::st(t68));
    let t69 = bh::ld(s.quad.matrix[1].bits());
    let t70 = t69 * bh::ld(s.up.bits());
    let t71 = t70 + bh::ld(s.quad.corners[3][0].bits());
    s.quad.corners[3][0].set_bits(bh::st(t71));
    let t72 = bh::ld(s.quad.matrix[4].bits());
    let t73 = t72 * bh::ld(s.up.bits());
    let t74 = t73 + bh::ld(s.quad.corners[3][1].bits());
    s.quad.corners[3][1].set_bits(bh::st(t74));
    let t75 = bh::ld(s.quad.matrix[7].bits());
    let t76 = t75 * bh::ld(s.up.bits());
    let t77 = t76 + bh::ld(s.quad.corners[3][2].bits());
    s.quad.corners[3][2].set_bits(bh::st(t77));
    let t78 = bh::ld(s.quad.corners[2][0].bits());
    let t79 = t78 - bh::ld(s.quad.corners[1][0].bits());
    s.quad.way[0].set_bits(bh::st(t79));
    let t80 = bh::ld(s.quad.corners[2][1].bits());
    let t81 = t80 - bh::ld(s.quad.corners[1][1].bits());
    s.quad.way[1].set_bits(bh::st(t81));
    let t82 = bh::ld(s.quad.corners[2][2].bits());
    let t83 = t82 - bh::ld(s.quad.corners[1][2].bits());
    s.quad.way[2].set_bits(bh::st(t83));
}

/// `corners_in`'s x87 block as plain Rust on the x87 model (45 instructions), same order and memory places, on either shadow's variables.
fn corners_in_x87(s: &Shadow) {
    let t0 = bh::ld(s.quad.way[0].bits());
    let t1 = t0 * bh::ld(s.quad.spread.bits());
    s.quad.way[0].set_bits(bh::st(t1));
    let t2 = bh::ld(s.quad.way[1].bits());
    let t3 = t2 * bh::ld(s.quad.spread.bits());
    s.quad.way[1].set_bits(bh::st(t3));
    let t4 = bh::ld(s.quad.way[2].bits());
    let t5 = t4 * bh::ld(s.quad.spread.bits());
    s.quad.way[2].set_bits(bh::st(t5));
    let t6 = bh::ld(s.quad.corners[0][0].bits());
    let t7 = t6 - bh::ld(s.quad.way[0].bits());
    s.quad.corners[0][0].set_bits(bh::st(t7));
    let t8 = bh::ld(s.quad.corners[0][1].bits());
    let t9 = t8 - bh::ld(s.quad.way[1].bits());
    s.quad.corners[0][1].set_bits(bh::st(t9));
    let t10 = bh::ld(s.quad.corners[0][2].bits());
    let t11 = t10 - bh::ld(s.quad.way[2].bits());
    s.quad.corners[0][2].set_bits(bh::st(t11));
    let t12 = bh::ld(s.quad.corners[1][0].bits());
    let t13 = t12 - bh::ld(s.quad.way[0].bits());
    s.quad.corners[1][0].set_bits(bh::st(t13));
    let t14 = bh::ld(s.quad.corners[1][1].bits());
    let t15 = t14 - bh::ld(s.quad.way[1].bits());
    s.quad.corners[1][1].set_bits(bh::st(t15));
    let t16 = bh::ld(s.quad.corners[1][2].bits());
    let t17 = t16 - bh::ld(s.quad.way[2].bits());
    s.quad.corners[1][2].set_bits(bh::st(t17));
    let t18 = bh::ld(s.quad.corners[2][0].bits());
    let t19 = t18 - bh::ld(s.quad.way[0].bits());
    s.quad.corners[2][0].set_bits(bh::st(t19));
    let t20 = bh::ld(s.quad.corners[2][1].bits());
    let t21 = t20 - bh::ld(s.quad.way[1].bits());
    s.quad.corners[2][1].set_bits(bh::st(t21));
    let t22 = bh::ld(s.quad.corners[2][2].bits());
    let t23 = t22 - bh::ld(s.quad.way[2].bits());
    s.quad.corners[2][2].set_bits(bh::st(t23));
    let t24 = bh::ld(s.quad.corners[3][0].bits());
    let t25 = t24 - bh::ld(s.quad.way[0].bits());
    s.quad.corners[3][0].set_bits(bh::st(t25));
    let t26 = bh::ld(s.quad.corners[3][1].bits());
    let t27 = t26 - bh::ld(s.quad.way[1].bits());
    s.quad.corners[3][1].set_bits(bh::st(t27));
    let t28 = bh::ld(s.quad.corners[3][2].bits());
    let t29 = t28 - bh::ld(s.quad.way[2].bits());
    s.quad.corners[3][2].set_bits(bh::st(t29));
}

/// `projected`'s x87 block as plain Rust on the x87 model (40 instructions), same order and memory places, on either shadow's variables.
fn projected_x87(s: &Shadow) {
    let t0 = crate::math::x87_float::ld(DRAW.view_scale.bits());
    let t1 = t0 / bh::ld(s.quad.corners[0][2].bits());
    let t2 = t1 * bh::ld(s.quad.corners[0][0].bits());
    let t3 = t2 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    s.quad.corners[0][0].set_bits(bh::st(t3));
    let t4 = t1 * bh::ld(s.quad.corners[0][1].bits());
    let t5 = -t4;
    let t6 = t5 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    s.quad.corners[0][1].set_bits(bh::st(t6));
    let t7 = crate::math::x87_float::ld(DRAW.view_scale.bits());
    let t8 = t7 / bh::ld(s.quad.corners[1][2].bits());
    let t9 = t8 * bh::ld(s.quad.corners[1][0].bits());
    let t10 = t9 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    s.quad.corners[1][0].set_bits(bh::st(t10));
    let t11 = t8 * bh::ld(s.quad.corners[1][1].bits());
    let t12 = -t11;
    let t13 = t12 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    s.quad.corners[1][1].set_bits(bh::st(t13));
    let t14 = crate::math::x87_float::ld(DRAW.view_scale.bits());
    let t15 = t14 / bh::ld(s.quad.corners[2][2].bits());
    let t16 = t15 * bh::ld(s.quad.corners[2][0].bits());
    let t17 = t16 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    s.quad.corners[2][0].set_bits(bh::st(t17));
    let t18 = t15 * bh::ld(s.quad.corners[2][1].bits());
    let t19 = -t18;
    let t20 = t19 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    s.quad.corners[2][1].set_bits(bh::st(t20));
    let t21 = crate::math::x87_float::ld(DRAW.view_scale.bits());
    let t22 = t21 / bh::ld(s.quad.corners[3][2].bits());
    let t23 = t22 * bh::ld(s.quad.corners[3][0].bits());
    let t24 = t23 + crate::math::x87_float::ld(DRAW.view_centre[0].bits());
    s.quad.corners[3][0].set_bits(bh::st(t24));
    let t25 = t22 * bh::ld(s.quad.corners[3][1].bits());
    let t26 = -t25;
    let t27 = t26 + crate::math::x87_float::ld(DRAW.view_centre[1].bits());
    s.quad.corners[3][1].set_bits(bh::st(t27));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn light_turned_x87(mk: usize, m: usize, t: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CFC74]",
            "fld dword ptr [{mk} + 0x1CFC78]",
            "fld dword ptr [{mk} + 0x1CFC7C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp dword ptr [{mk} + 0x1CEFA0]",
            "fstp dword ptr [{mk} + 0x1CEFA4]",
            "fstp dword ptr [{mk} + 0x1CEFA8]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn second_turned_x87(mk: usize, m: usize, t: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CEFA8]",
            "fld dword ptr [{mk} + 0x1CEFA4]",
            "fld dword ptr [{mk} + 0x1CEFA0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp dword ptr [{mk} + 0x1CF00C]",
            "fstp dword ptr [{mk} + 0x1CF010]",
            "fstp dword ptr [{mk} + 0x1CF014]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn corners_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CF000]",
            "fmul dword ptr [{mk} + 0x1CF028]",
            "fstp dword ptr [{mk} + 0x1CF000]",
            "fld dword ptr [{mk} + 0x1CF004]",
            "fmul dword ptr [{mk} + 0x1CF028]",
            "fstp dword ptr [{mk} + 0x1CF004]",
            "fld dword ptr [{mk} + 0x1CF008]",
            "fmul dword ptr [{mk} + 0x1CF028]",
            "fstp dword ptr [{mk} + 0x1CF008]",
            "fld dword ptr [{mk} + 0x1CF00C]",
            "fmul dword ptr [{mk} + 0x1CF02C]",
            "fstp dword ptr [{mk} + 0x1CF00C]",
            "fld dword ptr [{mk} + 0x1CF010]",
            "fmul dword ptr [{mk} + 0x1CF02C]",
            "fstp dword ptr [{mk} + 0x1CF010]",
            "fld dword ptr [{mk} + 0x1CF014]",
            "fmul dword ptr [{mk} + 0x1CF02C]",
            "fstp dword ptr [{mk} + 0x1CF014]",
            "fld dword ptr [{mk} + 0x1CEF90]",
            "fsub dword ptr [{mk} + 0x1CF000]",
            "fstp dword ptr [{mk} + 0x1CF058]",
            "fld dword ptr [{mk} + 0x1CEF94]",
            "fsub dword ptr [{mk} + 0x1CF004]",
            "fstp dword ptr [{mk} + 0x1CF05C]",
            "fld dword ptr [{mk} + 0x1CEF98]",
            "fsub dword ptr [{mk} + 0x1CF008]",
            "fstp dword ptr [{mk} + 0x1CF060]",
            "fld dword ptr [{mk} + 0x1CEF90]",
            "fadd dword ptr [{mk} + 0x1CF000]",
            "fstp dword ptr [{mk} + 0x1CF064]",
            "fld dword ptr [{mk} + 0x1CEF94]",
            "fadd dword ptr [{mk} + 0x1CF004]",
            "fstp dword ptr [{mk} + 0x1CF068]",
            "fld dword ptr [{mk} + 0x1CEF98]",
            "fadd dword ptr [{mk} + 0x1CF008]",
            "fstp dword ptr [{mk} + 0x1CF06C]",
            "fld dword ptr [{mk} + 0x1CEF90]",
            "fadd dword ptr [{mk} + 0x1CF000]",
            "fadd dword ptr [{mk} + 0x1CF00C]",
            "fstp dword ptr [{mk} + 0x1CF070]",
            "fld dword ptr [{mk} + 0x1CEF94]",
            "fadd dword ptr [{mk} + 0x1CF004]",
            "fadd dword ptr [{mk} + 0x1CF010]",
            "fstp dword ptr [{mk} + 0x1CF074]",
            "fld dword ptr [{mk} + 0x1CEF98]",
            "fadd dword ptr [{mk} + 0x1CF008]",
            "fadd dword ptr [{mk} + 0x1CF014]",
            "fstp dword ptr [{mk} + 0x1CF078]",
            "fld dword ptr [{mk} + 0x1CEF90]",
            "fsub dword ptr [{mk} + 0x1CF000]",
            "fadd dword ptr [{mk} + 0x1CF00C]",
            "fstp dword ptr [{mk} + 0x1CF07C]",
            "fld dword ptr [{mk} + 0x1CEF94]",
            "fsub dword ptr [{mk} + 0x1CF004]",
            "fadd dword ptr [{mk} + 0x1CF010]",
            "fstp dword ptr [{mk} + 0x1CF080]",
            "fld dword ptr [{mk} + 0x1CEF98]",
            "fsub dword ptr [{mk} + 0x1CF008]",
            "fadd dword ptr [{mk} + 0x1CF014]",
            "fstp dword ptr [{mk} + 0x1CF084]",
            "fld dword ptr [{mk} + 0x1CEFB0]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF058]",
            "fstp dword ptr [{mk} + 0x1CF058]",
            "fld dword ptr [{mk} + 0x1CEFBC]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF05C]",
            "fstp dword ptr [{mk} + 0x1CF05C]",
            "fld dword ptr [{mk} + 0x1CEFC8]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF060]",
            "fstp dword ptr [{mk} + 0x1CF060]",
            "fld dword ptr [{mk} + 0x1CEFB0]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF064]",
            "fstp dword ptr [{mk} + 0x1CF064]",
            "fld dword ptr [{mk} + 0x1CEFBC]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF068]",
            "fstp dword ptr [{mk} + 0x1CF068]",
            "fld dword ptr [{mk} + 0x1CEFC8]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF06C]",
            "fstp dword ptr [{mk} + 0x1CF06C]",
            "fld dword ptr [{mk} + 0x1CEFB0]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF070]",
            "fstp dword ptr [{mk} + 0x1CF070]",
            "fld dword ptr [{mk} + 0x1CEFBC]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF074]",
            "fstp dword ptr [{mk} + 0x1CF074]",
            "fld dword ptr [{mk} + 0x1CEFC8]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF078]",
            "fstp dword ptr [{mk} + 0x1CF078]",
            "fld dword ptr [{mk} + 0x1CEFB0]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF07C]",
            "fstp dword ptr [{mk} + 0x1CF07C]",
            "fld dword ptr [{mk} + 0x1CEFBC]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF080]",
            "fstp dword ptr [{mk} + 0x1CF080]",
            "fld dword ptr [{mk} + 0x1CEFC8]",
            "fmul dword ptr [{mk} + 0x1CFC38]",
            "fadd dword ptr [{mk} + 0x1CF084]",
            "fstp dword ptr [{mk} + 0x1CF084]",
            "fld dword ptr [{mk} + 0x1CF070]",
            "fsub dword ptr [{mk} + 0x1CF064]",
            "fstp dword ptr [{mk} + 0x1CF04C]",
            "fld dword ptr [{mk} + 0x1CF074]",
            "fsub dword ptr [{mk} + 0x1CF068]",
            "fstp dword ptr [{mk} + 0x1CF050]",
            "fld dword ptr [{mk} + 0x1CF078]",
            "fsub dword ptr [{mk} + 0x1CF06C]",
            "fstp dword ptr [{mk} + 0x1CF054]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn corners_in_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CF04C]",
            "fmul dword ptr [{mk} + 0x1CF048]",
            "fstp dword ptr [{mk} + 0x1CF04C]",
            "fld dword ptr [{mk} + 0x1CF050]",
            "fmul dword ptr [{mk} + 0x1CF048]",
            "fstp dword ptr [{mk} + 0x1CF050]",
            "fld dword ptr [{mk} + 0x1CF054]",
            "fmul dword ptr [{mk} + 0x1CF048]",
            "fstp dword ptr [{mk} + 0x1CF054]",
            "fld dword ptr [{mk} + 0x1CF058]",
            "fsub dword ptr [{mk} + 0x1CF04C]",
            "fstp dword ptr [{mk} + 0x1CF058]",
            "fld dword ptr [{mk} + 0x1CF05C]",
            "fsub dword ptr [{mk} + 0x1CF050]",
            "fstp dword ptr [{mk} + 0x1CF05C]",
            "fld dword ptr [{mk} + 0x1CF060]",
            "fsub dword ptr [{mk} + 0x1CF054]",
            "fstp dword ptr [{mk} + 0x1CF060]",
            "fld dword ptr [{mk} + 0x1CF064]",
            "fsub dword ptr [{mk} + 0x1CF04C]",
            "fstp dword ptr [{mk} + 0x1CF064]",
            "fld dword ptr [{mk} + 0x1CF068]",
            "fsub dword ptr [{mk} + 0x1CF050]",
            "fstp dword ptr [{mk} + 0x1CF068]",
            "fld dword ptr [{mk} + 0x1CF06C]",
            "fsub dword ptr [{mk} + 0x1CF054]",
            "fstp dword ptr [{mk} + 0x1CF06C]",
            "fld dword ptr [{mk} + 0x1CF070]",
            "fsub dword ptr [{mk} + 0x1CF04C]",
            "fstp dword ptr [{mk} + 0x1CF070]",
            "fld dword ptr [{mk} + 0x1CF074]",
            "fsub dword ptr [{mk} + 0x1CF050]",
            "fstp dword ptr [{mk} + 0x1CF074]",
            "fld dword ptr [{mk} + 0x1CF078]",
            "fsub dword ptr [{mk} + 0x1CF054]",
            "fstp dword ptr [{mk} + 0x1CF078]",
            "fld dword ptr [{mk} + 0x1CF07C]",
            "fsub dword ptr [{mk} + 0x1CF04C]",
            "fstp dword ptr [{mk} + 0x1CF07C]",
            "fld dword ptr [{mk} + 0x1CF080]",
            "fsub dword ptr [{mk} + 0x1CF050]",
            "fstp dword ptr [{mk} + 0x1CF080]",
            "fld dword ptr [{mk} + 0x1CF084]",
            "fsub dword ptr [{mk} + 0x1CF054]",
            "fstp dword ptr [{mk} + 0x1CF084]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn projected_x87(rc: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF060]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF058]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF058]",
            "fmul dword ptr [{mk} + 0x1CF05C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF05C]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF06C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF064]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF064]",
            "fmul dword ptr [{mk} + 0x1CF068]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF068]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF078]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF070]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF070]",
            "fmul dword ptr [{mk} + 0x1CF074]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF074]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CF084]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CF07C]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CF07C]",
            "fmul dword ptr [{mk} + 0x1CF080]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CF080]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn light_turned_2_x87(mk: usize, m: usize, t: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CEF74]",
            "fld dword ptr [{mk} + 0x1CEF78]",
            "fld dword ptr [{mk} + 0x1CEF7C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp dword ptr [{mk} + 0x1CE284]",
            "fstp dword ptr [{mk} + 0x1CE288]",
            "fstp dword ptr [{mk} + 0x1CE28C]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn second_turned_2_x87(mk: usize, m: usize, t: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CE28C]",
            "fld dword ptr [{mk} + 0x1CE288]",
            "fld dword ptr [{mk} + 0x1CE284]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0x0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x4]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x0]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC4",
            "fmul dword ptr [{m} + 0x14]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{t} + 0x4]",
            "fmul dword ptr [{m} + 0x18]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0x1C]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m} + 0x20]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fld dword ptr [{t} + 0x4]",
            "fld dword ptr [{t} + 0x0]",
            "fstp dword ptr [{mk} + 0x1CE2F0]",
            "fstp dword ptr [{mk} + 0x1CE2F4]",
            "fstp dword ptr [{mk} + 0x1CE2F8]",
            mk = in(reg) mk,
            m = in(reg) m,
            t = in(reg) t,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn corners_2_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CE2E4]",
            "fmul dword ptr [{mk} + 0x1CE30C]",
            "fstp dword ptr [{mk} + 0x1CE2E4]",
            "fld dword ptr [{mk} + 0x1CE2E8]",
            "fmul dword ptr [{mk} + 0x1CE30C]",
            "fstp dword ptr [{mk} + 0x1CE2E8]",
            "fld dword ptr [{mk} + 0x1CE2EC]",
            "fmul dword ptr [{mk} + 0x1CE30C]",
            "fstp dword ptr [{mk} + 0x1CE2EC]",
            "fld dword ptr [{mk} + 0x1CE2F0]",
            "fmul dword ptr [{mk} + 0x1CE310]",
            "fstp dword ptr [{mk} + 0x1CE2F0]",
            "fld dword ptr [{mk} + 0x1CE2F4]",
            "fmul dword ptr [{mk} + 0x1CE310]",
            "fstp dword ptr [{mk} + 0x1CE2F4]",
            "fld dword ptr [{mk} + 0x1CE2F8]",
            "fmul dword ptr [{mk} + 0x1CE310]",
            "fstp dword ptr [{mk} + 0x1CE2F8]",
            "fld dword ptr [{mk} + 0x1CE274]",
            "fsub dword ptr [{mk} + 0x1CE2E4]",
            "fstp dword ptr [{mk} + 0x1CE33C]",
            "fld dword ptr [{mk} + 0x1CE278]",
            "fsub dword ptr [{mk} + 0x1CE2E8]",
            "fstp dword ptr [{mk} + 0x1CE340]",
            "fld dword ptr [{mk} + 0x1CE27C]",
            "fsub dword ptr [{mk} + 0x1CE2EC]",
            "fstp dword ptr [{mk} + 0x1CE344]",
            "fld dword ptr [{mk} + 0x1CE274]",
            "fadd dword ptr [{mk} + 0x1CE2E4]",
            "fstp dword ptr [{mk} + 0x1CE348]",
            "fld dword ptr [{mk} + 0x1CE278]",
            "fadd dword ptr [{mk} + 0x1CE2E8]",
            "fstp dword ptr [{mk} + 0x1CE34C]",
            "fld dword ptr [{mk} + 0x1CE27C]",
            "fadd dword ptr [{mk} + 0x1CE2EC]",
            "fstp dword ptr [{mk} + 0x1CE350]",
            "fld dword ptr [{mk} + 0x1CE274]",
            "fadd dword ptr [{mk} + 0x1CE2E4]",
            "fadd dword ptr [{mk} + 0x1CE2F0]",
            "fstp dword ptr [{mk} + 0x1CE354]",
            "fld dword ptr [{mk} + 0x1CE278]",
            "fadd dword ptr [{mk} + 0x1CE2E8]",
            "fadd dword ptr [{mk} + 0x1CE2F4]",
            "fstp dword ptr [{mk} + 0x1CE358]",
            "fld dword ptr [{mk} + 0x1CE27C]",
            "fadd dword ptr [{mk} + 0x1CE2EC]",
            "fadd dword ptr [{mk} + 0x1CE2F8]",
            "fstp dword ptr [{mk} + 0x1CE35C]",
            "fld dword ptr [{mk} + 0x1CE274]",
            "fsub dword ptr [{mk} + 0x1CE2E4]",
            "fadd dword ptr [{mk} + 0x1CE2F0]",
            "fstp dword ptr [{mk} + 0x1CE360]",
            "fld dword ptr [{mk} + 0x1CE278]",
            "fsub dword ptr [{mk} + 0x1CE2E8]",
            "fadd dword ptr [{mk} + 0x1CE2F4]",
            "fstp dword ptr [{mk} + 0x1CE364]",
            "fld dword ptr [{mk} + 0x1CE27C]",
            "fsub dword ptr [{mk} + 0x1CE2EC]",
            "fadd dword ptr [{mk} + 0x1CE2F8]",
            "fstp dword ptr [{mk} + 0x1CE368]",
            "fld dword ptr [{mk} + 0x1CE294]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE33C]",
            "fstp dword ptr [{mk} + 0x1CE33C]",
            "fld dword ptr [{mk} + 0x1CE2A0]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE340]",
            "fstp dword ptr [{mk} + 0x1CE340]",
            "fld dword ptr [{mk} + 0x1CE2AC]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE344]",
            "fstp dword ptr [{mk} + 0x1CE344]",
            "fld dword ptr [{mk} + 0x1CE294]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE348]",
            "fstp dword ptr [{mk} + 0x1CE348]",
            "fld dword ptr [{mk} + 0x1CE2A0]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE34C]",
            "fstp dword ptr [{mk} + 0x1CE34C]",
            "fld dword ptr [{mk} + 0x1CE2AC]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE350]",
            "fstp dword ptr [{mk} + 0x1CE350]",
            "fld dword ptr [{mk} + 0x1CE294]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE354]",
            "fstp dword ptr [{mk} + 0x1CE354]",
            "fld dword ptr [{mk} + 0x1CE2A0]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE358]",
            "fstp dword ptr [{mk} + 0x1CE358]",
            "fld dword ptr [{mk} + 0x1CE2AC]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE35C]",
            "fstp dword ptr [{mk} + 0x1CE35C]",
            "fld dword ptr [{mk} + 0x1CE294]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE360]",
            "fstp dword ptr [{mk} + 0x1CE360]",
            "fld dword ptr [{mk} + 0x1CE2A0]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE364]",
            "fstp dword ptr [{mk} + 0x1CE364]",
            "fld dword ptr [{mk} + 0x1CE2AC]",
            "fmul dword ptr [{mk} + 0x1CEF30]",
            "fadd dword ptr [{mk} + 0x1CE368]",
            "fstp dword ptr [{mk} + 0x1CE368]",
            "fld dword ptr [{mk} + 0x1CE354]",
            "fsub dword ptr [{mk} + 0x1CE348]",
            "fstp dword ptr [{mk} + 0x1CE330]",
            "fld dword ptr [{mk} + 0x1CE358]",
            "fsub dword ptr [{mk} + 0x1CE34C]",
            "fstp dword ptr [{mk} + 0x1CE334]",
            "fld dword ptr [{mk} + 0x1CE35C]",
            "fsub dword ptr [{mk} + 0x1CE350]",
            "fstp dword ptr [{mk} + 0x1CE338]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn corners_in_2_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1CE330]",
            "fmul dword ptr [{mk} + 0x1CE32C]",
            "fstp dword ptr [{mk} + 0x1CE330]",
            "fld dword ptr [{mk} + 0x1CE334]",
            "fmul dword ptr [{mk} + 0x1CE32C]",
            "fstp dword ptr [{mk} + 0x1CE334]",
            "fld dword ptr [{mk} + 0x1CE338]",
            "fmul dword ptr [{mk} + 0x1CE32C]",
            "fstp dword ptr [{mk} + 0x1CE338]",
            "fld dword ptr [{mk} + 0x1CE33C]",
            "fsub dword ptr [{mk} + 0x1CE330]",
            "fstp dword ptr [{mk} + 0x1CE33C]",
            "fld dword ptr [{mk} + 0x1CE340]",
            "fsub dword ptr [{mk} + 0x1CE334]",
            "fstp dword ptr [{mk} + 0x1CE340]",
            "fld dword ptr [{mk} + 0x1CE344]",
            "fsub dword ptr [{mk} + 0x1CE338]",
            "fstp dword ptr [{mk} + 0x1CE344]",
            "fld dword ptr [{mk} + 0x1CE348]",
            "fsub dword ptr [{mk} + 0x1CE330]",
            "fstp dword ptr [{mk} + 0x1CE348]",
            "fld dword ptr [{mk} + 0x1CE34C]",
            "fsub dword ptr [{mk} + 0x1CE334]",
            "fstp dword ptr [{mk} + 0x1CE34C]",
            "fld dword ptr [{mk} + 0x1CE350]",
            "fsub dword ptr [{mk} + 0x1CE338]",
            "fstp dword ptr [{mk} + 0x1CE350]",
            "fld dword ptr [{mk} + 0x1CE354]",
            "fsub dword ptr [{mk} + 0x1CE330]",
            "fstp dword ptr [{mk} + 0x1CE354]",
            "fld dword ptr [{mk} + 0x1CE358]",
            "fsub dword ptr [{mk} + 0x1CE334]",
            "fstp dword ptr [{mk} + 0x1CE358]",
            "fld dword ptr [{mk} + 0x1CE35C]",
            "fsub dword ptr [{mk} + 0x1CE338]",
            "fstp dword ptr [{mk} + 0x1CE35C]",
            "fld dword ptr [{mk} + 0x1CE360]",
            "fsub dword ptr [{mk} + 0x1CE330]",
            "fstp dword ptr [{mk} + 0x1CE360]",
            "fld dword ptr [{mk} + 0x1CE364]",
            "fsub dword ptr [{mk} + 0x1CE334]",
            "fstp dword ptr [{mk} + 0x1CE364]",
            "fld dword ptr [{mk} + 0x1CE368]",
            "fsub dword ptr [{mk} + 0x1CE338]",
            "fstp dword ptr [{mk} + 0x1CE368]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn projected_2_x87(rc: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE344]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE33C]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE33C]",
            "fmul dword ptr [{mk} + 0x1CE340]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE340]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE350]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE348]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE348]",
            "fmul dword ptr [{mk} + 0x1CE34C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE34C]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE35C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE354]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE354]",
            "fmul dword ptr [{mk} + 0x1CE358]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE358]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CE368]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CE360]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CE360]",
            "fmul dword ptr [{mk} + 0x1CE364]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CE364]",
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
    use super::Shadow;
    use super::asm_reference as reference;
    use crate::effects::asm_test::with_copy;
    use crate::effects::kind_store::{KindConsts, QuadWork};
    use crate::effects::store::START;
    use crate::session::Float;

    /// A shadow's variables: the quad, the constants and the up (IDA addresses).
    #[derive(Clone, Copy)]
    struct Layout(usize, usize, usize, u32);
    const SET_1: Layout = Layout(0x9331B4, 0x933E7C, 0x933E70, 0x26A0);
    const SET_2: Layout = Layout(0x933ED0, 0x934B7C, 0x934B78, 0x26A4);

    /// `f` on copies of a shadow's variables in the test buffer standing for the marks'
    /// block at `mk`, put back afterwards.
    fn with_shadow<R>(mk: usize, l: Layout, f: impl FnOnce(&Shadow) -> R) -> R {
        with_copy::<QuadWork, R>(mk, l.0, |quad| {
            with_copy::<KindConsts, R>(mk, l.1, |consts| {
                with_copy::<Float, R>(mk, l.2, |up| {
                    f(&Shadow {
                        quad,
                        consts,
                        up,
                        field: if l.3 == 0x26A0 {
                            |marks| &marks.listener_float_a
                        } else {
                            |marks| &marks.listener_float_b
                        },
                    })
                })
            })
        })
    }
    /// `f` on the matrix and the two temporaries at the test addresses `m` and `t`.
    fn with_matrix(m: usize, t: usize, f: impl FnOnce(&[Float; 9], &[Float; 2])) {
        with_copy::<[Float; 9], ()>(m, START, |m| {
            with_copy::<[Float; 2], ()>(t, START, |t| f(m, t));
        });
    }
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, differential};

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
        differential(
            &SPEC,
            |b| reference::light_turned_x87(b[0], b[1], b[2]),
            |b| {
                with_shadow(b[0], SET_2, |s| {
                    with_matrix(b[1], b[2], |m, t| super::light_turned_x87(s, m, t));
                });
            },
        );
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
        differential(
            &SPEC,
            |b| reference::second_turned_x87(b[0], b[1], b[2]),
            |b| {
                with_shadow(b[0], SET_2, |s| {
                    with_matrix(b[1], b[2], |m, t| super::second_turned_x87(s, m, t));
                });
            },
        );
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
        differential(
            &SPEC,
            |b| reference::corners_x87(b[0]),
            |b| with_shadow(b[0], SET_2, super::corners_x87),
        );
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
        differential(
            &SPEC,
            |b| reference::corners_in_x87(b[0]),
            |b| with_shadow(b[0], SET_2, super::corners_in_x87),
        );
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
        differential(
            &SPEC,
            |b| reference::projected_x87(b[0], b[1]),
            |b| with_shadow(b[1], SET_2, super::projected_x87),
        );
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
        differential(
            &SPEC,
            |b| reference::light_turned_2_x87(b[0], b[1], b[2]),
            |b| {
                with_shadow(b[0], SET_1, |s| {
                    with_matrix(b[1], b[2], |m, t| super::light_turned_x87(s, m, t));
                });
            },
        );
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
        differential(
            &SPEC,
            |b| reference::second_turned_2_x87(b[0], b[1], b[2]),
            |b| {
                with_shadow(b[0], SET_1, |s| {
                    with_matrix(b[1], b[2], |m, t| super::second_turned_x87(s, m, t));
                });
            },
        );
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
        differential(
            &SPEC,
            |b| reference::corners_2_x87(b[0]),
            |b| with_shadow(b[0], SET_1, super::corners_x87),
        );
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
        differential(
            &SPEC,
            |b| reference::corners_in_2_x87(b[0]),
            |b| with_shadow(b[0], SET_1, super::corners_in_x87),
        );
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
        differential(
            &SPEC,
            |b| reference::projected_2_x87(b[0], b[1]),
            |b| with_shadow(b[1], SET_1, super::projected_x87),
        );
    }
}
