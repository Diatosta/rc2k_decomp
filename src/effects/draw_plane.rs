//! `effects::draw_plane` (0x4C8F64): a quad laid on the windscreen drops' frame (a reading: the
//! ground glare): the rate, the place and two axes from the settings (+0x2720 .., +0x272C ..,
//! +0x2708 .., +0x2740), their lengths' shares, made one long and square
//! (`scene::normalise_vector3`, `scene::cross_product3`), scaled (`effects::scale_axes`);
//! nothing more unless the place's way faces the third axis away, or unless
//! `dword_641678`; `scene::depth_test_off`, render states 0x11 = 2, 0x12 = 2,
//! `scene::set_texture` (`dword_5D1E10`); the four corners; the texture places from corner
//! `dword_934C64` of `g_marksCorners`, then from `flt_934F3C` ..; colour `g_viewLight`;
//! unless a corner's depth is at or below `flt_5C5E18` (signed compares of the bits): the
//! corners projected, their depth (plus `g_marksKindFloat` and `flt_934F5C` .., times
//! `flt_8EB96D`) and rhw (1 over the depth times `flt_934F6C` ..), `scene::draw_fan`,
//! `scene::depth_test_on`.
//!
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::draw_overlays` (0x4C752F). The x87 runs are plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the game's variables; the vector helpers and
//! `scale_axes` their Rust versions. Refuses null settings and a corner past `g_marksCorners`,
//! before anything.

use crate::effects::kind_store::QuadWork;
use crate::effects::plain::ldi_cell;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::marks_settings_store::MarksSettings;
use crate::scene::cross_product3::cross;
use crate::scene::vector3::normalise;

use crate::d3d::vertices_store::VERTICES;
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

/// The plane's constants (IDA 0x934F30 .. 0x934F7C; phase 7, typed state). The offsets are from its start.
#[repr(C)]
pub struct PlaneTuning {
    /// +0x0: what the settings' rate is multiplied by into the first axis' scale (`flt_934F30`)
    pub rate_k: f32,
    /// +0x4: what the two axes' lengths are divided by (`flt_934F34`, `flt_934F38`)
    pub shares: [f32; 2],
    /// +0xC: the four corners' texture places (u, v) (`flt_934F3C`, `flt_934F40` ..)
    pub places: [[f32; 2]; 4],
    /// +0x2C: what each corner's depth is raised by (`flt_934F5C`, `flt_934F60` ..)
    pub lifts: [f32; 4],
    /// +0x3C: what one over each corner's depth is multiplied by (`flt_934F6C`, `flt_934F70` ..)
    pub rhw_k: [f32; 4],
}

/// The plane's constants; the first values are the image's.
pub static PLANE_TUNING: PlaneTuning = PlaneTuning {
    rate_k: f32::from_bits(0x3F800000),
    shares: [f32::from_bits(0x45000000), f32::from_bits(0x44800000)],
    places: [
        [f32::from_bits(0x0), f32::from_bits(0x3F008000)],
        [f32::from_bits(0x3F7FF972), f32::from_bits(0x3F008000)],
        [f32::from_bits(0x3F7FF972); 2],
        [f32::from_bits(0x0), f32::from_bits(0x3F7FF972)],
    ],
    lifts: [f32::from_bits(0x43FA0000); 4],
    rhw_k: [f32::from_bits(0x43800000); 4],
};

#[cfg(all(test, target_arch = "x86"))]
impl PlaneTuning {
    pub const START: usize = 0x934F30;
}

/// The original's 0x4C8F6A .. 0x4C9000: the rate (`flt_89CA34`), the place (`flt_89CEF6` ..) and the first axis (`flt_89CF1A` ..) from the settings, its length over `flt_934F34` into `flt_89CEEE`.
fn plane_axes(fl: &WindscreenFlakes, m: &MarksSettings) {
    plane_axes_x87(m, &fl.timing, &fl.tables, &PLANE_TUNING);
}

/// The original's 0x4C900D .. 0x4C906D: the second axis (`flt_89CF26` ..), its length over `flt_934F38` into `flt_89CEF2`.
fn plane_up(fl: &WindscreenFlakes, m: &MarksSettings) {
    plane_up_x87(m, &fl.tables, &PLANE_TUNING);
}

/// The original's 0x4C90BE .. 0x4C90E2: the place as floats into `flt_8161A2` ...
fn plane_place(drops: &WindscreenDrops, m: &MarksSettings) {
    plane_place_x87(drops, m);
}

/// The original's 0x4C90ED .. 0x4C9113: the place's way dotted with the third axis into `flt_814718`.
fn plane_facing(drops: &WindscreenDrops, fl: &WindscreenFlakes) {
    plane_facing_x87(drops, &fl.tables);
}

/// The original's 0x4C915A .. 0x4C9244: the plane's centre and corners (`flt_934C98` .. `flt_934CC4`) from the axes.
fn plane_corners(fl: &WindscreenFlakes, effects: &crate::effects::owner::Effects) {
    plane_corners_x87(&fl.tables, &effects.kinds.quad_3);
}

/// The original's 0x4C93FA .. 0x4C94CA: the four corners projected.
fn plane_projected(effects: &crate::effects::owner::Effects, scene: &crate::scene::owner::Scene) {
    plane_projected_x87(&effects.kinds.quad_3, scene);
}

/// The Rust version (see the header).
pub fn draw_plane(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    use crate::scene::set_render_state::set_render_state;
    let m = crate::effects::setup::marks()?;
    let corner: u32 = effects.kinds.quad_3.corner.get();
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
    if drops.state.facing.bits() & 0x8000_0000 == 0 {
        return Ok(());
    }
    if race.frame.frame_work.lead_kept[7].bits() == 0 {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a plane corner past g_marksCorners").into());
    }
    crate::scene::depth_test_off::depth_test_off();
    for state in [0x11, 0x12] {
        set_render_state(2, state);
    }
    crate::scene::set_texture::set_texture(scene.textures.kinds.kind_first_slots[202].bits());
    plane_corners(fl, effects);
    let entry = &effects.blocks.corners.corners[corner as usize];
    let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
    let uv_k: u32 = effects.frame.draw_work.uv_k.bits();
    for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
        let vertex = &VERTICES.vertices[n];
        vertex.u.set_bits(x87::int_times(word(u) as i32, uv_k));
        vertex.v.set_bits(x87::int_times(word(v) as i32, uv_k));
    }
    for n in 0..4 {
        let vertex = &VERTICES.vertices[n];
        vertex
            .u
            .set_bits(x87::loaded(PLANE_TUNING.places[n][0].to_bits()));
        vertex
            .v
            .set_bits(x87::loaded(PLANE_TUNING.places[n][1].to_bits()));
    }
    let light: u32 = ctx.view.light.view_light.bits();
    for n in 0..4 {
        VERTICES.vertices[n].colour.set(light);
    }
    for n in 0..4 {
        if (effects.kinds.quad_3.corners[n][2].bits() as i32)
            <= (race.course.numbers.k4096_a.bits() as i32)
        {
            return Ok(());
        }
    }
    plane_projected(effects, scene);
    let [kind, z_k]: [u32; 2] = [
        effects.setup.kind_floats.kind_float.bits(),
        effects.frame.draw_work.z_k.bits(),
    ];
    for n in 0..4 {
        let c = &effects.kinds.quad_3.corners[n];
        let vertex = &VERTICES.vertices[n];
        vertex.x.set_bits(c[0].bits());
        vertex.y.set_bits(c[1].bits());
        let depth: u32 = c[2].bits();
        vertex.z.set_bits(x87::sum3_times(
            depth,
            kind,
            PLANE_TUNING.lifts[n].to_bits(),
            z_k,
        ));
    }
    for n in 0..4 {
        let depth: u32 = effects.kinds.quad_3.corners[n][2].bits();
        VERTICES.vertices[n].rhw.set_bits(x87::reciprocal_times(
            depth,
            PLANE_TUNING.rhw_k[n].to_bits(),
        ));
    }
    crate::effects::clip_quad::draw_fan();
    crate::scene::depth_test_on::depth_test_on();
    Ok(())
}

use crate::effects::plain::{ld_cell, st_cell};
use crate::effects::windscreen_flakes_store::{WindscreenFlakeTables, WindscreenFlakeTiming};
use crate::math::x87_float as bh;

/// `plane_axes`'s x87 block as plain Rust on the x87 model (29 instructions), same order and memory places.
fn plane_axes_x87(
    m: &MarksSettings,
    sti: &WindscreenFlakeTiming,
    sq: &WindscreenFlakeTables,
    pt: &PlaneTuning,
) {
    let t0 = ldi_cell(&m.listener_switch_3);
    let t1 = t0 * bh::ld(pt.rate_k.to_bits());
    sti.rate.set_bits(bh::st(t1));
    let t2 = ldi_cell(&m.listener_point_b_negated[0]);
    let t3 = ldi_cell(&m.listener_point_b_negated[1]);
    let t4 = ldi_cell(&m.listener_point_b_negated[2]);
    sq.place[2].set_bits(bh::st(t4));
    sq.place[1].set_bits(bh::st(t3));
    sq.place[0].set_bits(bh::st(t2));
    let t5 = ldi_cell(&m.listener_point_b[0]);
    let t6 = t5 - ldi_cell(&m.listener_point_b_negated[0]);
    sq.axes[0][0].set_bits(bh::st(t6));
    let t7 = ldi_cell(&m.listener_point_b[1]);
    let t8 = t7 - ldi_cell(&m.listener_point_b_negated[1]);
    sq.axes[0][1].set_bits(bh::st(t8));
    let t9 = ldi_cell(&m.listener_point_b[2]);
    let t10 = t9 - ldi_cell(&m.listener_point_b_negated[2]);
    sq.axes[0][2].set_bits(bh::st(t10));
    let t11 = bh::ld(sq.axes[0][0].bits());
    let t12 = t11 * t11;
    let t13 = bh::ld(sq.axes[0][1].bits());
    let t14 = t13 * t13;
    let t15 = bh::ld(sq.axes[0][2].bits());
    let t16 = t15 * t15;
    let t17 = t14 + t16;
    let t18 = t12 + t17;
    let t19 = t18.sqrt();
    let t20 = t19 / bh::ld(pt.shares[0].to_bits());
    sq.lengths[0].set_bits(bh::st(t20));
}

/// `plane_up`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
fn plane_up_x87(m: &MarksSettings, sq: &WindscreenFlakeTables, pt: &PlaneTuning) {
    let t0 = ldi_cell(&m.listener_point_a_negated[0]);
    let t1 = t0 - ldi_cell(&m.listener_point_b_negated[0]);
    sq.axes[1][0].set_bits(bh::st(t1));
    let t2 = ldi_cell(&m.listener_point_a_negated[1]);
    let t3 = t2 - ldi_cell(&m.listener_point_b_negated[1]);
    sq.axes[1][1].set_bits(bh::st(t3));
    let t4 = ldi_cell(&m.listener_point_a_negated[2]);
    let t5 = t4 - ldi_cell(&m.listener_point_b_negated[2]);
    sq.axes[1][2].set_bits(bh::st(t5));
    let t6 = bh::ld(sq.axes[1][0].bits());
    let t7 = t6 * t6;
    let t8 = bh::ld(sq.axes[1][1].bits());
    let t9 = t8 * t8;
    let t10 = bh::ld(sq.axes[1][2].bits());
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 / bh::ld(pt.shares[1].to_bits());
    sq.lengths[1].set_bits(bh::st(t15));
}

/// `plane_place`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
fn plane_place_x87(drops: &WindscreenDrops, m: &MarksSettings) {
    let t0 = ldi_cell(&m.listener_point_b_negated[0]);
    st_cell(&drops.hex.place[0], t0);
    let t1 = ldi_cell(&m.listener_point_b_negated[1]);
    st_cell(&drops.hex.place[1], t1);
    let t2 = ldi_cell(&m.listener_point_b_negated[2]);
    st_cell(&drops.hex.place[2], t2);
}

/// `plane_facing`'s x87 block as plain Rust on the x87 model (9 instructions), same order and memory places.
fn plane_facing_x87(drops: &WindscreenDrops, sq: &WindscreenFlakeTables) {
    let t0 = ld_cell(&drops.hex.place[0]);
    let t1 = t0 * bh::ld(sq.axes[2][0].bits());
    let t2 = ld_cell(&drops.hex.place[1]);
    let t3 = t2 * bh::ld(sq.axes[2][1].bits());
    let t4 = ld_cell(&drops.hex.place[2]);
    let t5 = t4 * bh::ld(sq.axes[2][2].bits());
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    drops.state.facing.set_bits(bh::st(t7));
}

/// `plane_corners`'s x87 block as plain Rust on the x87 model (47 instructions), same order and memory places.
fn plane_corners_x87(sq: &WindscreenFlakeTables, q: &QuadWork) {
    let t0 = bh::X(0.0);
    let t1 = bh::X(0.0);
    let t2 = t1 * bh::ld(sq.lengths[1].bits());
    let t3 = t0 * bh::ld(sq.lengths[0].bits());
    let t4 = t3 * bh::ld(sq.axes[0][0].bits());
    let t5 = t2 * bh::ld(sq.axes[1][0].bits());
    let t6 = t4 + t5;
    let t7 = t6 + bh::ld(sq.place[0].bits());
    q.corners[3][0].set_bits(bh::st(t7));
    let t8 = t3 * bh::ld(sq.axes[0][1].bits());
    let t9 = t2 * bh::ld(sq.axes[1][1].bits());
    let t10 = t8 + t9;
    let t11 = t10 + bh::ld(sq.place[1].bits());
    q.corners[3][1].set_bits(bh::st(t11));
    let t12 = t2 * bh::ld(sq.axes[1][2].bits());
    let t13 = t3 * bh::ld(sq.axes[0][2].bits());
    let t14 = t12 + t13;
    let t15 = t14 + bh::ld(sq.place[2].bits());
    q.corners[3][2].set_bits(bh::st(t15));
    let t16 = bh::ld(q.corners[3][0].bits());
    let t17 = t16 + bh::ld(sq.scaled_axes[1][0].bits());
    q.corners[0][0].set_bits(bh::st(t17));
    let t18 = t17 + bh::ld(sq.scaled_axes[0][0].bits());
    q.corners[1][0].set_bits(bh::st(t18));
    let t19 = t18 - bh::ld(sq.scaled_axes[1][0].bits());
    q.corners[2][0].set_bits(bh::st(t19));
    let t20 = bh::ld(q.corners[3][1].bits());
    let t21 = t20 + bh::ld(sq.scaled_axes[1][1].bits());
    q.corners[0][1].set_bits(bh::st(t21));
    let t22 = t21 + bh::ld(sq.scaled_axes[0][1].bits());
    q.corners[1][1].set_bits(bh::st(t22));
    let t23 = t22 - bh::ld(sq.scaled_axes[1][1].bits());
    q.corners[2][1].set_bits(bh::st(t23));
    let t24 = bh::ld(q.corners[3][2].bits());
    let t25 = t24 + bh::ld(sq.scaled_axes[1][2].bits());
    q.corners[0][2].set_bits(bh::st(t25));
    let t26 = t25 + bh::ld(sq.scaled_axes[0][2].bits());
    q.corners[1][2].set_bits(bh::st(t26));
    let t27 = t26 - bh::ld(sq.scaled_axes[1][2].bits());
    q.corners[2][2].set_bits(bh::st(t27));
}

/// `plane_projected`'s x87 block as plain Rust on the x87 model (40 instructions), same order and memory places.
fn plane_projected_x87(q: &QuadWork, scene: &crate::scene::owner::Scene) {
    let t0 = crate::math::x87_float::ld(scene.drawing.scene.view_scale.bits());
    let t1 = t0 / bh::ld(q.corners[0][2].bits());
    let t2 = t1 * bh::ld(q.corners[0][0].bits());
    let t3 = t2 + crate::math::x87_float::ld(scene.drawing.scene.view_centre[0].bits());
    q.corners[0][0].set_bits(bh::st(t3));
    let t4 = t1 * bh::ld(q.corners[0][1].bits());
    let t5 = -t4;
    let t6 = t5 + crate::math::x87_float::ld(scene.drawing.scene.view_centre[1].bits());
    q.corners[0][1].set_bits(bh::st(t6));
    let t7 = crate::math::x87_float::ld(scene.drawing.scene.view_scale.bits());
    let t8 = t7 / bh::ld(q.corners[1][2].bits());
    let t9 = t8 * bh::ld(q.corners[1][0].bits());
    let t10 = t9 + crate::math::x87_float::ld(scene.drawing.scene.view_centre[0].bits());
    q.corners[1][0].set_bits(bh::st(t10));
    let t11 = t8 * bh::ld(q.corners[1][1].bits());
    let t12 = -t11;
    let t13 = t12 + crate::math::x87_float::ld(scene.drawing.scene.view_centre[1].bits());
    q.corners[1][1].set_bits(bh::st(t13));
    let t14 = crate::math::x87_float::ld(scene.drawing.scene.view_scale.bits());
    let t15 = t14 / bh::ld(q.corners[2][2].bits());
    let t16 = t15 * bh::ld(q.corners[2][0].bits());
    let t17 = t16 + crate::math::x87_float::ld(scene.drawing.scene.view_centre[0].bits());
    q.corners[2][0].set_bits(bh::st(t17));
    let t18 = t15 * bh::ld(q.corners[2][1].bits());
    let t19 = -t18;
    let t20 = t19 + crate::math::x87_float::ld(scene.drawing.scene.view_centre[1].bits());
    q.corners[2][1].set_bits(bh::st(t20));
    let t21 = crate::math::x87_float::ld(scene.drawing.scene.view_scale.bits());
    let t22 = t21 / bh::ld(q.corners[3][2].bits());
    let t23 = t22 * bh::ld(q.corners[3][0].bits());
    let t24 = t23 + crate::math::x87_float::ld(scene.drawing.scene.view_centre[0].bits());
    q.corners[3][0].set_bits(bh::st(t24));
    let t25 = t22 * bh::ld(q.corners[3][1].bits());
    let t26 = -t25;
    let t27 = t26 + crate::math::x87_float::ld(scene.drawing.scene.view_centre[1].bits());
    q.corners[3][1].set_bits(bh::st(t27));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn plane_axes_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{s} + 0x2740]",
            "fmul dword ptr [{mk} + 0x1CFFF0]",
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
            "fdiv dword ptr [{mk} + 0x1CFFF4]",
            "fstp dword ptr [{mk} + 0x137FAE]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn plane_up_x87(mk: usize, s: usize) {
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
            "fdiv dword ptr [{mk} + 0x1CFFF8]",
            "fstp dword ptr [{mk} + 0x137FB2]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn plane_place_x87(mk: usize, s: usize) {
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

    pub(super) fn plane_facing_x87(mk: usize) {
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

    pub(super) fn plane_corners_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    ".byte 0xD9, 0xEE",
            ".byte 0xD9, 0xEE",
            "fmul dword ptr [{mk} + 0x137FB2]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x137FAE]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FDA]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FE6]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FB6]",
            "fstp dword ptr [{mk} + 0x1CFD7C]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FDE]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FEA]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FBA]",
            "fstp dword ptr [{mk} + 0x1CFD80]",
            "fmul dword ptr [{mk} + 0x137FEE]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x137FE2]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FBE]",
            "fstp dword ptr [{mk} + 0x1CFD84]",
            "fld dword ptr [{mk} + 0x1CFD7C]",
            "fadd dword ptr [{mk} + 0x13803A]",
            "fst dword ptr [{mk} + 0x1CFD58]",
            "fadd dword ptr [{mk} + 0x13802E]",
            "fst dword ptr [{mk} + 0x1CFD64]",
            "fsub dword ptr [{mk} + 0x13803A]",
            "fstp dword ptr [{mk} + 0x1CFD70]",
            "fld dword ptr [{mk} + 0x1CFD80]",
            "fadd dword ptr [{mk} + 0x13803E]",
            "fst dword ptr [{mk} + 0x1CFD5C]",
            "fadd dword ptr [{mk} + 0x138032]",
            "fst dword ptr [{mk} + 0x1CFD68]",
            "fsub dword ptr [{mk} + 0x13803E]",
            "fstp dword ptr [{mk} + 0x1CFD74]",
            "fld dword ptr [{mk} + 0x1CFD84]",
            "fadd dword ptr [{mk} + 0x138042]",
            "fst dword ptr [{mk} + 0x1CFD60]",
            "fadd dword ptr [{mk} + 0x138036]",
            "fst dword ptr [{mk} + 0x1CFD6C]",
            "fsub dword ptr [{mk} + 0x138042]",
            "fstp dword ptr [{mk} + 0x1CFD78]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn plane_projected_x87(rc: usize, mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CFD60]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CFD58]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CFD58]",
            "fmul dword ptr [{mk} + 0x1CFD5C]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CFD5C]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CFD6C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CFD64]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CFD64]",
            "fmul dword ptr [{mk} + 0x1CFD68]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CFD68]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CFD78]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CFD70]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CFD70]",
            "fmul dword ptr [{mk} + 0x1CFD74]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CFD74]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1CFD84]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1CFD7C]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1CFD7C]",
            "fmul dword ptr [{mk} + 0x1CFD80]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1CFD80]",
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
    use super::{PlaneTuning, QuadWork};
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::with_copy;
    use crate::effects::asm_test::with_settings;
    use crate::effects::asm_test::{Op, Spec, differential};
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
        differential(
            &SPEC,
            |b| reference::plane_axes_x87(b[0], b[1]),
            |b| {
                with_copy::<WindscreenFlakeTiming, _>(b[0], WindscreenFlakeTiming::START, |sti| {
                    with_copy::<WindscreenFlakeTables, _>(
                        b[0],
                        WindscreenFlakeTables::START,
                        |sq| {
                            with_copy::<PlaneTuning, _>(b[0], PlaneTuning::START, |pt| {
                                with_settings(b[1], |m| super::plane_axes_x87(m, sti, sq, pt));
                            });
                        },
                    );
                });
            },
        );
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
        differential(
            &SPEC,
            |b| reference::plane_up_x87(b[0], b[1]),
            |b| {
                with_copy::<WindscreenFlakeTables, _>(b[0], WindscreenFlakeTables::START, |sq| {
                    with_copy::<PlaneTuning, _>(b[0], PlaneTuning::START, |pt| {
                        with_settings(b[1], |m| super::plane_up_x87(m, sq, pt));
                    });
                });
            },
        );
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
        differential(
            &SPEC,
            |b| reference::plane_place_x87(b[0], b[1]),
            |b| {
                with_overlays(
                    b[0],
                    &[overlay(
                        &crate::effects::owner::EFFECTS.windscreen_drops.hex,
                        WindscreenDropVars::START,
                    )],
                    || {
                        with_settings(b[1], |m| {
                            super::plane_place_x87(
                                &crate::effects::owner::EFFECTS.windscreen_drops,
                                m,
                            );
                        });
                    },
                );
            },
        );
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
        differential(
            &SPEC,
            |b| reference::plane_facing_x87(b[0]),
            |b| {
                with_overlays(
                    b[0],
                    &[
                        overlay(
                            &crate::effects::owner::EFFECTS.windscreen_drops.state,
                            WindscreenDropState::START,
                        ),
                        overlay(
                            &crate::effects::owner::EFFECTS.windscreen_drops.hex,
                            WindscreenDropVars::START,
                        ),
                    ],
                    || {
                        with_copy::<WindscreenFlakeTables, _>(
                            b[0],
                            WindscreenFlakeTables::START,
                            |m| {
                                super::plane_facing_x87(
                                    &crate::effects::owner::EFFECTS.windscreen_drops,
                                    m,
                                );
                            },
                        );
                    },
                );
            },
        );
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
        differential(
            &SPEC,
            |b| reference::plane_corners_x87(b[0]),
            |b| {
                with_copy::<WindscreenFlakeTables, _>(b[0], WindscreenFlakeTables::START, |sq| {
                    with_copy::<QuadWork, _>(b[0], 0x934BD0, |q| super::plane_corners_x87(sq, q));
                });
            },
        );
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
        differential(
            &SPEC,
            |b| reference::plane_projected_x87(b[0], b[1]),
            |b| {
                with_copy::<QuadWork, _>(b[1], 0x934BD0, |q| {
                    super::plane_projected_x87(q, crate::scene::owner::scene());
                });
            },
        );
    }
}
