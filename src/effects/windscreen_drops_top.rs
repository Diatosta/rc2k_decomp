//! `effects::windscreen_drops_top` (0x4BAC87): the windscreen drops' frame from the settings: the vertex
//! ring from its start (`dword_816232` = `unk_625A44`); the ease and the rate; the place
//! (the settings' +0x2720 ..) and two axes (+0x272C .. and +0x2708 .. less it), their
//! lengths' shares (`flt_814C06`, `flt_814C0A`), made one long and square
//! (`scene::normalise_vector3`, `scene::cross_product3`); nothing more unless the place's way
//! faces the third axis away (a negative dot, `flt_814718`); the 0x30 points of
//! `g_marksWindscreenDropPoints` scaled into `flt_814CFE` .. and turned onto the axes; unless the
//! rate is 0: `effects::windscreen_drops_frame`, the vertices left in the ring drawn (their count by
//! `dword_82E27A`: `dword_62423C` / `dword_625A40`, `sub_41ADB9`, the platform's), and with
//! the settings' +0x2744 and the drift on `effects::windscreen_drops_step`; then `scene::set_texture`
//! (`dword_5D1E10`) and `g_polygonVertexCount` 4.
//!
//! Screen: the rain drops on the windscreen in the in-car view (owner, 2026-10-29: gone with `d3d::draw_marks_windscreen_drops` cut). Called the hexagons until then (their shape).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4C74DA` (0x4C7539). The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables; the vector helpers their Rust versions. Refuses null settings, before
//! anything.

use crate::effects::plain::ldi_cell;
use crate::effects::setup::{CLOCK, MarksClock};
use crate::effects::tuning::MARKS_KIND_FLOATS;
use crate::effects::windscreen_drop_state::{WINDSCREEN_DROP_STATE, WINDSCREEN_DROP_WORK};
use crate::effects::windscreen_drop_vars::HEX;
use crate::errors::Refused;
use crate::gfx::windscreen_drop_batch::WINDSCREEN_DROP_BATCH;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::marks_settings_store::MarksSettings;
use crate::scene::cross_product3::cross;
use crate::scene::draw_store::DRAW;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::scene::vector3::normalise;

const POINTS: u32 = 0x30;

/// The original's 0x4BAC91 .. 0x4BAD3F: `flt_81622A` (the ease by the ticks), `flt_814728` (the rate), the place (`flt_814C0E` ..) and the first axis (`flt_814C1A` ..) from the settings, its length over `flt_82E272` into `flt_814C06`.
fn frame_axes(m: &MarksSettings) {
    frame_axes_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        m,
        &CLOCK,
    );
}

/// The original's 0x4BAD4C .. 0x4BADAC: the second axis (`flt_814C26` ..) from the settings, its length over `flt_82E276` into `flt_814C0A`.
fn frame_up(m: &MarksSettings) {
    frame_up_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        m,
    );
}

/// The original's 0x4BADF8 .. 0x4BAE1C: the place again as floats into `flt_8161A2` ...
fn frame_place(m: &MarksSettings) {
    frame_place_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        m,
    );
}

/// The original's 0x4BAE27 .. 0x4BAE4D: the place's way dotted with the third axis into `flt_814718`.
fn frame_facing() {
    frame_facing_x87(crate::effects::windscreen_drop_vars::WindscreenDropVars::base());
}

/// The original's 0x4BAEAB .. 0x4BAEEC: a point (x, y at p) turned onto the axes (x, y, z back at p).
fn point_turned(p: u32) {
    point_turned_x87(crate::effects::store::base() as usize, p as usize);
}

/// The Rust version (see the header).
pub fn windscreen_drops_top() -> Result<(), Refused> {
    let m = crate::effects::setup::marks()?;
    HEX.buffer.set(0);
    frame_axes(m);
    let w = &WINDSCREEN_DROP_WORK.axes;
    let (a0, a1, a2) = (&w[0..3], &w[3..6], &w[6..9]);
    normalise(a0);
    frame_up(m);
    normalise(a1);
    cross(a1, a0, a2);
    normalise(a2);
    cross(a0, a2, a1);
    frame_place(m);
    normalise(&HEX.place);
    frame_facing();
    if WINDSCREEN_DROP_STATE.facing.bits() & 0x8000_0000 == 0 {
        return Ok(());
    }
    let [kx, ky] = WINDSCREEN_DROP_WORK
        .shares
        .each_ref()
        .map(super::super::session::Float::bits);
    for n in 0..POINTS as usize {
        let (p, o) = (3 * n, 3 * n);
        WINDSCREEN_DROP_WORK.scaled[o]
            .set_bits(x87::mul(WINDSCREEN_DROP_WORK.points[p].bits(), kx));
        WINDSCREEN_DROP_WORK.scaled[o + 1]
            .set_bits(x87::mul(WINDSCREEN_DROP_WORK.points[p + 1].bits(), ky));
        WINDSCREEN_DROP_WORK.scaled[o + 2].set_bits(0);
    }
    for n in 0..POINTS {
        point_turned(WINDSCREEN_DROP_WORK.scaled[0].addr() + 12 * n);
    }
    if WINDSCREEN_DROP_STATE.rate.bits() != 0 {
        crate::effects::windscreen_drops_frame::windscreen_drops_frame()?;
        let at_now: u32 = HEX.buffer.get();
        if at_now != 0 {
            let quads = x87::int_over_int_to_int(at_now as i32, HEX.per_vertex.get() as i32) as u32;
            HEX.buffer.set(0);
            WINDSCREEN_DROP_BATCH
                .index_count
                .set(quads.wrapping_mul(12));
            WINDSCREEN_DROP_BATCH
                .vertex_count
                .set(quads.wrapping_mul(6));
            crate::effects::windscreen_drops_frame::draw_batch()?;
        }
        let marks = crate::effects::setup::marks()?;
        if marks.listener_switch_6.get() != 0 && MARKS_KIND_FLOATS.running.bits() != 0 {
            crate::effects::windscreen_drops_step::windscreen_drops_step()?;
        }
    }
    crate::scene::set_texture::set_texture(KIND_TEXTURES.kind_first_slots[202].bits());
    DRAW.polygon_vertex_count.set_bits(4u32);
    Ok(())
}

use crate::effects::plain::{ld, st};

/// `frame_axes`'s x87 block as plain Rust on the x87 model (32 instructions), same order and memory places.
fn frame_axes_x87(hx: usize, m: &MarksSettings, c: &MarksClock) {
    let t0 = ld(hx, 0xC932A);
    let t1 = t0 * crate::math::x87_float::int(c.frame_ticks.get() as i32);
    st(hx, 0xB12EA, t1);
    let t2 = ldi_cell(&m.listener_switch_0);
    let t3 = t2 * ld(hx, 0xC932E);
    WINDSCREEN_DROP_STATE.rate.set_bits(bh::st(t3));
    let t4 = ldi_cell(&m.listener_point_b_negated[0]);
    let t5 = ldi_cell(&m.listener_point_b_negated[1]);
    let t6 = ldi_cell(&m.listener_point_b_negated[2]);
    WINDSCREEN_DROP_WORK.place[2].set_bits(bh::st(t6));
    WINDSCREEN_DROP_WORK.place[1].set_bits(bh::st(t5));
    WINDSCREEN_DROP_WORK.place[0].set_bits(bh::st(t4));
    let t7 = ldi_cell(&m.listener_point_b[0]);
    let t8 = t7 - ldi_cell(&m.listener_point_b_negated[0]);
    WINDSCREEN_DROP_WORK.axes[0].set_bits(bh::st(t8));
    let t9 = ldi_cell(&m.listener_point_b[1]);
    let t10 = t9 - ldi_cell(&m.listener_point_b_negated[1]);
    WINDSCREEN_DROP_WORK.axes[1].set_bits(bh::st(t10));
    let t11 = ldi_cell(&m.listener_point_b[2]);
    let t12 = t11 - ldi_cell(&m.listener_point_b_negated[2]);
    WINDSCREEN_DROP_WORK.axes[2].set_bits(bh::st(t12));
    let t13 = bh::ld(WINDSCREEN_DROP_WORK.axes[0].bits());
    let t14 = t13 * t13;
    let t15 = bh::ld(WINDSCREEN_DROP_WORK.axes[1].bits());
    let t16 = t15 * t15;
    let t17 = bh::ld(WINDSCREEN_DROP_WORK.axes[2].bits());
    let t18 = t17 * t17;
    let t19 = t16 + t18;
    let t20 = t14 + t19;
    let t21 = t20.sqrt();
    let t22 = t21 / ld(hx, 0xC9332);
    WINDSCREEN_DROP_WORK.shares[0].set_bits(bh::st(t22));
}

/// `frame_up`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
fn frame_up_x87(hx: usize, m: &MarksSettings) {
    let t0 = ldi_cell(&m.listener_point_a_negated[0]);
    let t1 = t0 - ldi_cell(&m.listener_point_b_negated[0]);
    WINDSCREEN_DROP_WORK.axes[3].set_bits(bh::st(t1));
    let t2 = ldi_cell(&m.listener_point_a_negated[1]);
    let t3 = t2 - ldi_cell(&m.listener_point_b_negated[1]);
    WINDSCREEN_DROP_WORK.axes[4].set_bits(bh::st(t3));
    let t4 = ldi_cell(&m.listener_point_a_negated[2]);
    let t5 = t4 - ldi_cell(&m.listener_point_b_negated[2]);
    WINDSCREEN_DROP_WORK.axes[5].set_bits(bh::st(t5));
    let t6 = bh::ld(WINDSCREEN_DROP_WORK.axes[3].bits());
    let t7 = t6 * t6;
    let t8 = bh::ld(WINDSCREEN_DROP_WORK.axes[4].bits());
    let t9 = t8 * t8;
    let t10 = bh::ld(WINDSCREEN_DROP_WORK.axes[5].bits());
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 / ld(hx, 0xC9336);
    WINDSCREEN_DROP_WORK.shares[1].set_bits(bh::st(t15));
}

/// `frame_place`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
fn frame_place_x87(hx: usize, m: &MarksSettings) {
    let t0 = ldi_cell(&m.listener_point_b_negated[0]);
    st(hx, 0xB1262, t0);
    let t1 = ldi_cell(&m.listener_point_b_negated[1]);
    st(hx, 0xB1266, t1);
    let t2 = ldi_cell(&m.listener_point_b_negated[2]);
    st(hx, 0xB126A, t2);
}

/// `frame_facing`'s x87 block as plain Rust on the x87 model (9 instructions), same order and memory places.
fn frame_facing_x87(hx: usize) {
    let t0 = ld(hx, 0xB1262);
    let t1 = t0 * bh::ld(WINDSCREEN_DROP_WORK.axes[6].bits());
    let t2 = ld(hx, 0xB1266);
    let t3 = t2 * bh::ld(WINDSCREEN_DROP_WORK.axes[7].bits());
    let t4 = ld(hx, 0xB126A);
    let t5 = t4 * bh::ld(WINDSCREEN_DROP_WORK.axes[8].bits());
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    WINDSCREEN_DROP_STATE.facing.set_bits(bh::st(t7));
}

/// `point_turned`'s x87 block as plain Rust on the x87 model (19 instructions), same order and memory places.
fn point_turned_x87(_mk: usize, p: usize) {
    let t0 = ld(p, 0);
    let t1 = ld(p, 0x4);
    let t2 = t0 * bh::ld(WINDSCREEN_DROP_WORK.axes[0].bits());
    let t3 = t1 * bh::ld(WINDSCREEN_DROP_WORK.axes[3].bits());
    let t4 = t2 + t3;
    st(p, 0, t4);
    let t5 = t0 * bh::ld(WINDSCREEN_DROP_WORK.axes[1].bits());
    let t6 = t1 * bh::ld(WINDSCREEN_DROP_WORK.axes[4].bits());
    let t7 = t5 + t6;
    st(p, 0x4, t7);
    let t8 = t1 * bh::ld(WINDSCREEN_DROP_WORK.axes[5].bits());
    let t9 = t0 * bh::ld(WINDSCREEN_DROP_WORK.axes[2].bits());
    let t10 = t8 + t9;
    st(p, 0x8, t10);
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
    use crate::effects::windscreen_drop_state::{
        WINDSCREEN_DROP_STATE, WINDSCREEN_DROP_WORK, WindscreenDropState, WindscreenDropWork,
    };

    #[test]
    fn frame_axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x90,
                    hi: 0xC9336,
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
                        overlay(&WINDSCREEN_DROP_STATE, WindscreenDropState::START),
                        overlay(&WINDSCREEN_DROP_WORK, WindscreenDropWork::START),
                    ],
                    || {
                        with_settings(b[1], |m| {
                            super::frame_axes_x87(b[0], m, crate::effects::setup::clock_at(b[0]));
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
                    hi: 0xC933A,
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
                    &[overlay(&WINDSCREEN_DROP_WORK, WindscreenDropWork::START)],
                    || with_settings(b[1], |m| super::frame_up_x87(b[0], m)),
                );
            },
        );
    }

    #[test]
    fn frame_place_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xB1262,
                    hi: 0xB126E,
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
            |b| with_settings(b[1], |m| super::frame_place_x87(b[0], m)),
        );
    }

    #[test]
    fn frame_facing_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xAF7D0,
                hi: 0xB126E,
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
                        overlay(&WINDSCREEN_DROP_STATE, WindscreenDropState::START),
                        overlay(&WINDSCREEN_DROP_WORK, WindscreenDropWork::START),
                    ],
                    || super::frame_facing_x87(b[0]),
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
                    &[overlay(&WINDSCREEN_DROP_WORK, WindscreenDropWork::START)],
                    || super::point_turned_x87(b[0], b[1]),
                );
            },
        );
    }
}
