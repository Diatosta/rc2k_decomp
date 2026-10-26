//! `marks::draw_plane` (0x4C8F64): a quad laid on the hexagons' frame (a reading: the
//! ground glare): the rate, the place and two axes from the settings (+0x2720 .., +0x272C ..,
//! +0x2708 .., +0x2740), their lengths' shares, made one long and square
//! (`scene::normalise_vector3`, `scene::cross_product3`), scaled (`marks::scale_axes`);
//! nothing more unless the place's way faces the third axis away, or unless
//! `dword_641678`; `scene::depth_test_off`, render states 0x11 = 2, 0x12 = 2,
//! `scene::set_texture` (`dword_5D1E10`); the four corners; the texture places from corner
//! `dword_934C64` of `g_marksCorners`, then from `flt_934F3C` ..; colour `g_viewLight`;
//! unless a corner's depth is at or below `flt_5C5E18` (signed compares of the bits): the
//! corners projected, their depth (plus `g_marksKindFloat` and `flt_934F5C` .., times
//! `flt_8EB96D`) and rhw (1 over the depth times `flt_934F6C` ..), `scene::draw_fan`,
//! `scene::depth_test_on`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4C74DA` (0x4C752F). The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables; the vector helpers and `scale_axes` their Rust versions. Refuses null
//! settings and a corner past `g_marksCorners`, before anything.

use crate::errors::{BadInput, Refused};
use crate::marks::blocks::MARK_CORNERS;
use crate::marks::frame_state::DRAW_WORK;
use crate::marks::hexagon_state::HEXAGON_STATE;
use crate::marks::hexagon_vars::HEX;
use crate::marks::kind_store::{QUAD_3, QuadWork};
use crate::marks::squares_store::SQUARE_TABLES;
use crate::marks::tuning::MARKS_KIND_FLOATS;
use crate::math::x87;
use crate::race::frame_store::FRAME_WORK;
use crate::race::numbers_store::NUMBERS;
use crate::scene::cross_product3::cross_product3;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::scene::vector3::normalise_vector3;
use crate::session::{Float, address_of};

use crate::d3d::vertices_store::VERTICES;
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

/// The plane's constants (IDA 0x934F30 .. 0x934F7C; phase 7, typed state). The offsets are from its start.
#[repr(C)]
pub struct PlaneTuning {
    /// +0x0: what the settings' rate is multiplied by into the first axis' scale (`flt_934F30`)
    pub rate_k: Float,
    /// +0x4: what the two axes' lengths are divided by (`flt_934F34`, `flt_934F38`)
    pub shares: [Float; 2],
    /// +0xC: the four corners' texture places (u, v) (`flt_934F3C`, `flt_934F40` ..)
    pub places: [[Float; 2]; 4],
    /// +0x2C: what each corner's depth is raised by (`flt_934F5C`, `flt_934F60` ..)
    pub lifts: [Float; 4],
    /// +0x3C: what one over each corner's depth is multiplied by (`flt_934F6C`, `flt_934F70` ..)
    pub rhw_k: [Float; 4],
}

/// The plane's constants; the first values are the image's.
pub static PLANE_TUNING: PlaneTuning = PlaneTuning {
    rate_k: Float::from_bits(0x3F800000),
    shares: [Float::from_bits(0x45000000), Float::from_bits(0x44800000)],
    places: [
        [Float::from_bits(0x0), Float::from_bits(0x3F008000)],
        [Float::from_bits(0x3F7FF972), Float::from_bits(0x3F008000)],
        [const { Float::from_bits(0x3F7FF972) }; 2],
        [Float::from_bits(0x0), Float::from_bits(0x3F7FF972)],
    ],
    lifts: [const { Float::from_bits(0x43FA0000) }; 4],
    rhw_k: [const { Float::from_bits(0x43800000) }; 4],
};

impl PlaneTuning {
    pub const START: usize = 0x934F30;
    pub const LEN: usize = 0x4C;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&PLANE_TUNING as *const PlaneTuning) as usize
    }
}

/// The original's 0x4C8F6A .. 0x4C9000: the rate (`flt_89CA34`), the place (`flt_89CEF6` ..) and the first axis (`flt_89CF1A` ..) from the settings, its length over `flt_934F34` into `flt_89CEEE`.
fn plane_axes(s: u32) {
    plane_axes_x87(s as usize, &SQUARE_TIMING, &SQUARE_TABLES, &PLANE_TUNING)
}

/// The original's 0x4C900D .. 0x4C906D: the second axis (`flt_89CF26` ..), its length over `flt_934F38` into `flt_89CEF2`.
fn plane_up(s: u32) {
    plane_up_x87(s as usize, &SQUARE_TABLES, &PLANE_TUNING)
}

/// The original's 0x4C90BE .. 0x4C90E2: the place as floats into `flt_8161A2` ...
fn plane_place(s: u32) {
    plane_place_x87(crate::marks::hexagon_vars::HexagonVars::base(), s as usize)
}

/// The original's 0x4C90ED .. 0x4C9113: the place's way dotted with the third axis into `flt_814718`.
fn plane_facing() {
    plane_facing_x87(
        crate::marks::hexagon_vars::HexagonVars::base(),
        &SQUARE_TABLES,
    )
}

/// The original's 0x4C915A .. 0x4C9244: the plane's centre and corners (`flt_934C98` .. `flt_934CC4`) from the axes.
fn plane_corners() {
    plane_corners_x87(&SQUARE_TABLES, &QUAD_3)
}

/// The original's 0x4C93FA .. 0x4C94CA: the four corners projected.
fn plane_projected() {
    plane_projected_x87(crate::race::store::asm_base() as usize, &QUAD_3)
}

/// The Rust version (see the header).
pub fn draw_plane() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    let s: u32 = crate::marks::setup::SETTINGS.pointer.get();
    if s == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let corner: u32 = QUAD_3.corner.get();
    plane_axes(s);
    let [a0, a1, a2] = [0, 1, 2].map(|n| address_of(&SQUARE_TABLES.axes[n]));
    normalise_vector3(a0);
    plane_up(s);
    normalise_vector3(a1);
    cross_product3(a1, a0, a2)?;
    normalise_vector3(a2);
    cross_product3(a0, a2, a1)?;
    crate::marks::scale_axes::scale_axes();
    plane_place(s);
    normalise_vector3(crate::pointers::game_address(HEX.place.as_ptr()));
    plane_facing();
    if HEXAGON_STATE.facing.bits() & 0x8000_0000 == 0 {
        return Ok(());
    }
    if FRAME_WORK.lead_kept[7].bits() == 0 {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a plane corner past g_marksCorners").into());
    }
    crate::scene::depth_test_off::depth_test_off();
    for state in [0x11, 0x12] {
        set_render_state(2, state);
    }
    crate::scene::set_texture::set_texture(KIND_TEXTURES.kind_first_slots[202].bits());
    plane_corners();
    let entry = &MARK_CORNERS.corners[corner as usize];
    let word = |off: u32| u32::from(entry.words[off as usize / 2].get());
    let uv_k: u32 = DRAW_WORK.uv_k.bits();
    for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
        let vertex = &VERTICES.vertices[n];
        vertex.u.set_bits(x87::int_times(word(u) as i32, uv_k));
        vertex.v.set_bits(x87::int_times(word(v) as i32, uv_k));
    }
    for n in 0..4 {
        let vertex = &VERTICES.vertices[n];
        vertex
            .u
            .set_bits(x87::loaded(PLANE_TUNING.places[n][0].bits()));
        vertex
            .v
            .set_bits(x87::loaded(PLANE_TUNING.places[n][1].bits()));
    }
    let light: u32 = FRAME_WORK.view_light.bits();
    for n in 0..4 {
        VERTICES.vertices[n].colour.set(light);
    }
    for n in 0..4 {
        if (QUAD_3.corners[n][2].bits() as i32) <= (NUMBERS.k4096_a.bits() as i32) {
            return Ok(());
        }
    }
    plane_projected();
    let [kind, z_k]: [u32; 2] = [MARKS_KIND_FLOATS.kind_float.bits(), DRAW_WORK.z_k.bits()];
    for n in 0..4 {
        let c = &QUAD_3.corners[n];
        let vertex = &VERTICES.vertices[n];
        vertex.x.set_bits(c[0].bits());
        vertex.y.set_bits(c[1].bits());
        let depth: u32 = c[2].bits();
        vertex.z.set_bits(x87::sum3_times(
            depth,
            kind,
            PLANE_TUNING.lifts[n].bits(),
            z_k,
        ));
    }
    for n in 0..4 {
        let depth: u32 = QUAD_3.corners[n][2].bits();
        VERTICES.vertices[n]
            .rhw
            .set_bits(x87::reciprocal_times(depth, PLANE_TUNING.rhw_k[n].bits()));
    }
    crate::marks::clip_quad::draw_fan();
    crate::scene::depth_test_on::depth_test_on();
    Ok(())
}

use crate::marks::plain::{ld, ldi, st};
use crate::marks::squares_store::{SQUARE_TIMING, SquareTables, SquareTiming};
use crate::math::x87_float as bh;

/// `plane_axes`'s x87 block as plain Rust on the x87 model (29 instructions), same order and memory places.
fn plane_axes_x87(s: usize, sti: &SquareTiming, sq: &SquareTables, pt: &PlaneTuning) {
    let t0 = ldi(s, 0x2740);
    let t1 = t0 * bh::ld(pt.rate_k.bits());
    sti.rate.set_bits(bh::st(t1));
    let t2 = ldi(s, 0x2720);
    let t3 = ldi(s, 0x2724);
    let t4 = ldi(s, 0x2728);
    sq.place[2].set_bits(bh::st(t4));
    sq.place[1].set_bits(bh::st(t3));
    sq.place[0].set_bits(bh::st(t2));
    let t5 = ldi(s, 0x272C);
    let t6 = t5 - ldi(s, 0x2720);
    sq.axes[0][0].set_bits(bh::st(t6));
    let t7 = ldi(s, 0x2730);
    let t8 = t7 - ldi(s, 0x2724);
    sq.axes[0][1].set_bits(bh::st(t8));
    let t9 = ldi(s, 0x2734);
    let t10 = t9 - ldi(s, 0x2728);
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
    let t20 = t19 / bh::ld(pt.shares[0].bits());
    sq.lengths[0].set_bits(bh::st(t20));
}

/// `plane_up`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
fn plane_up_x87(s: usize, sq: &SquareTables, pt: &PlaneTuning) {
    let t0 = ldi(s, 0x2708);
    let t1 = t0 - ldi(s, 0x2720);
    sq.axes[1][0].set_bits(bh::st(t1));
    let t2 = ldi(s, 0x270C);
    let t3 = t2 - ldi(s, 0x2724);
    sq.axes[1][1].set_bits(bh::st(t3));
    let t4 = ldi(s, 0x2710);
    let t5 = t4 - ldi(s, 0x2728);
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
    let t15 = t14 / bh::ld(pt.shares[1].bits());
    sq.lengths[1].set_bits(bh::st(t15));
}

/// `plane_place`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
fn plane_place_x87(hx: usize, s: usize) {
    let t0 = ldi(s, 0x2720);
    st(hx, 0xB1262, t0);
    let t1 = ldi(s, 0x2724);
    st(hx, 0xB1266, t1);
    let t2 = ldi(s, 0x2728);
    st(hx, 0xB126A, t2);
}

/// `plane_facing`'s x87 block as plain Rust on the x87 model (9 instructions), same order and memory places.
fn plane_facing_x87(hx: usize, sq: &SquareTables) {
    let t0 = ld(hx, 0xB1262);
    let t1 = t0 * bh::ld(sq.axes[2][0].bits());
    let t2 = ld(hx, 0xB1266);
    let t3 = t2 * bh::ld(sq.axes[2][1].bits());
    let t4 = ld(hx, 0xB126A);
    let t5 = t4 * bh::ld(sq.axes[2][2].bits());
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    HEXAGON_STATE.facing.set_bits(bh::st(t7));
}

/// `plane_corners`'s x87 block as plain Rust on the x87 model (47 instructions), same order and memory places.
fn plane_corners_x87(sq: &SquareTables, q: &QuadWork) {
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
fn plane_projected_x87(rc: usize, q: &QuadWork) {
    let t0 = ld(rc, 0x224088);
    let t1 = t0 / bh::ld(q.corners[0][2].bits());
    let t2 = t1 * bh::ld(q.corners[0][0].bits());
    let t3 = t2 + ld(rc, 0x22409C);
    q.corners[0][0].set_bits(bh::st(t3));
    let t4 = t1 * bh::ld(q.corners[0][1].bits());
    let t5 = -t4;
    let t6 = t5 + ld(rc, 0x2240A0);
    q.corners[0][1].set_bits(bh::st(t6));
    let t7 = ld(rc, 0x224088);
    let t8 = t7 / bh::ld(q.corners[1][2].bits());
    let t9 = t8 * bh::ld(q.corners[1][0].bits());
    let t10 = t9 + ld(rc, 0x22409C);
    q.corners[1][0].set_bits(bh::st(t10));
    let t11 = t8 * bh::ld(q.corners[1][1].bits());
    let t12 = -t11;
    let t13 = t12 + ld(rc, 0x2240A0);
    q.corners[1][1].set_bits(bh::st(t13));
    let t14 = ld(rc, 0x224088);
    let t15 = t14 / bh::ld(q.corners[2][2].bits());
    let t16 = t15 * bh::ld(q.corners[2][0].bits());
    let t17 = t16 + ld(rc, 0x22409C);
    q.corners[2][0].set_bits(bh::st(t17));
    let t18 = t15 * bh::ld(q.corners[2][1].bits());
    let t19 = -t18;
    let t20 = t19 + ld(rc, 0x2240A0);
    q.corners[2][1].set_bits(bh::st(t20));
    let t21 = ld(rc, 0x224088);
    let t22 = t21 / bh::ld(q.corners[3][2].bits());
    let t23 = t22 * bh::ld(q.corners[3][0].bits());
    let t24 = t23 + ld(rc, 0x22409C);
    q.corners[3][0].set_bits(bh::st(t24));
    let t25 = t22 * bh::ld(q.corners[3][1].bits());
    let t26 = -t25;
    let t27 = t26 + ld(rc, 0x2240A0);
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
            )
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
            )
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
            )
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
            )
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
            )
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
            )
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use super::{PlaneTuning, QuadWork};
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::with_copy;
    use crate::marks::asm_test::{Op, Spec, differential};
    use crate::marks::hexagon_state::{HEXAGON_STATE, HexagonState};
    use crate::marks::squares_store::{SquareTables, SquareTiming};
    use crate::marks::typed_sync::{overlay, with_overlays};

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
                    lo: 0x2720,
                    hi: 0x2744,
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
                with_copy::<SquareTiming, _>(b[0], SquareTiming::START, |sti| {
                    with_copy::<SquareTables, _>(b[0], SquareTables::START, |sq| {
                        with_copy::<PlaneTuning, _>(b[0], PlaneTuning::START, |pt| {
                            super::plane_axes_x87(b[1], sti, sq, pt)
                        })
                    })
                })
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
                    lo: 0x2708,
                    hi: 0x272C,
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
                with_copy::<SquareTables, _>(b[0], SquareTables::START, |sq| {
                    with_copy::<PlaneTuning, _>(b[0], PlaneTuning::START, |pt| {
                        super::plane_up_x87(b[1], sq, pt)
                    })
                })
            },
        );
    }

    #[test]
    fn plane_place_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xB1262,
                    hi: 0xB126E,
                    reads: &[],
                    writes: &[(0xB1262, F32), (0xB1266, F32), (0xB126A, F32)],
                },
                Op::Base {
                    lo: 0x2720,
                    hi: 0x272C,
                    reads: &[(0x2720, I32), (0x2724, I32), (0x2728, I32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::plane_place_x87(b[0], b[1]),
            |b| super::plane_place_x87(b[0], b[1]),
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
                    &[overlay(&HEXAGON_STATE, HexagonState::START)],
                    || {
                        with_copy::<SquareTables, _>(b[0], SquareTables::START, |sq| {
                            super::plane_facing_x87(b[0], sq)
                        })
                    },
                )
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
                with_copy::<SquareTables, _>(b[0], SquareTables::START, |sq| {
                    with_copy::<QuadWork, _>(b[0], 0x934BD0, |q| super::plane_corners_x87(sq, q))
                })
            },
        );
    }

    #[test]
    fn plane_projected_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x224088,
                    hi: 0x2240A4,
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
            |b| with_copy::<QuadWork, _>(b[1], 0x934BD0, |q| super::plane_projected_x87(b[0], q)),
        );
    }
}
