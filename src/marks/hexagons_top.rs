//! `marks::hexagons_top` (0x4BAC87): the hexagons' frame from the settings: the vertex
//! ring from its start (`dword_816232` = `unk_625A44`); the ease and the rate; the place
//! (the settings' +0x2720 ..) and two axes (+0x272C .. and +0x2708 .. less it), their
//! lengths' shares (`flt_814C06`, `flt_814C0A`), made one long and square
//! (`scene::normalise_vector3`, `scene::cross_product3`); nothing more unless the place's way
//! faces the third axis away (a negative dot, `flt_814718`); the 0x30 points of
//! `g_marksHexagonPoints` scaled into `flt_814CFE` .. and turned onto the axes; unless the
//! rate is 0: `marks::hexagons_frame`, the vertices left in the ring drawn (their count by
//! `dword_82E27A`: `dword_62423C` / `dword_625A40`, `sub_41ADB9`, the platform's), and with
//! the settings' +0x2744 and the drift on `marks::hexagons_step`; then `scene::set_texture`
//! (`dword_5D1E10`) and `g_polygonVertexCount` 4.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4C74DA` (0x4C7539). The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables; the vector helpers their Rust versions. Refuses null settings, before
//! anything.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::gfx::hexagon_batch::HEXAGON_BATCH;
use crate::marks::setup::{CLOCK, MarksClock};
use crate::marks::store::{Slot, Where, game_addr, game_read, game_write};
use crate::math::x87;
use crate::scene::cross_product3::cross_product3;
use crate::scene::vector3::normalise_vector3;

const BUFFER_RVA: Slot = Slot::at(0x816232); // dword_816232
const AXES: [Slot; 3] = [Slot::at(0x814C1A), Slot::at(0x814C26), Slot::at(0x814C32)];
const PLACE_RVA: Slot = Slot::at(0x8161A2); // flt_8161A2
const FACING_RVA: Slot = Slot::at(0x814718); // flt_814718
const POINTS_RVA: Slot = Slot::at(0x814F3E); // g_marksHexagonPoints: 0x30 of 12 bytes
const SCALED_RVA: Slot = Slot::at(0x814CFE); // flt_814CFE ..
const POINTS: u32 = 0x30;
const SHARES: [Slot; 2] = [Slot::at(0x814C06), Slot::at(0x814C0A)];
const RATE_RVA: Slot = Slot::at(0x814728); // flt_814728
const PER_VERTEX_RVA: Slot = Slot::at(0x82E27A); // dword_82E27A
const DRIFTING_RVA: Slot = Slot::at(0x7652A0); // dword_7652A0
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount

fn addr(r: impl Where) -> u32 {
    game_addr(r) as u32
}

/// The original's 0x4BAC91 .. 0x4BAD3F: `flt_81622A` (the ease by the ticks), `flt_814728` (the rate), the place (`flt_814C0E` ..) and the first axis (`flt_814C1A` ..) from the settings, its length over `flt_82E272` into `flt_814C06`.
unsafe fn frame_axes(s: u32) {
    unsafe { frame_axes_x87(crate::marks::store::base() as usize, s as usize, &CLOCK) }
}

/// The original's 0x4BAD4C .. 0x4BADAC: the second axis (`flt_814C26` ..) from the settings, its length over `flt_82E276` into `flt_814C0A`.
unsafe fn frame_up(s: u32) {
    unsafe { frame_up_x87(crate::marks::store::base() as usize, s as usize) }
}

/// The original's 0x4BADF8 .. 0x4BAE1C: the place again as floats into `flt_8161A2` ...
unsafe fn frame_place(s: u32) {
    unsafe { frame_place_x87(crate::marks::store::base() as usize, s as usize) }
}

/// The original's 0x4BAE27 .. 0x4BAE4D: the place's way dotted with the third axis into `flt_814718`.
unsafe fn frame_facing() {
    unsafe { frame_facing_x87(crate::marks::store::base() as usize) }
}

/// The original's 0x4BAEAB .. 0x4BAEEC: a point (x, y at p) turned onto the axes (x, y, z back at p).
unsafe fn point_turned(p: u32) {
    unsafe { point_turned_x87(crate::marks::store::base() as usize, p as usize) }
}

/// The Rust version (see the header).
pub unsafe fn hexagons_top() -> Result<(), Refused> {
    unsafe {
        let s: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        game_write(BUFFER_RVA, HEXAGON_BATCH.records.addr() as u32);
        frame_axes(s);
        let [a0, a1, a2] = AXES.map(addr);
        normalise_vector3(a0);
        frame_up(s);
        normalise_vector3(a1);
        cross_product3(a1, a0, a2)?;
        normalise_vector3(a2);
        cross_product3(a0, a2, a1)?;
        frame_place(s);
        normalise_vector3(addr(PLACE_RVA));
        frame_facing();
        if game_read::<u32>(FACING_RVA) & 0x8000_0000 == 0 {
            return Ok(());
        }
        let [kx, ky] = SHARES.map(|r| game_read::<u32>(r));
        for n in 0..POINTS as usize {
            let p = POINTS_RVA + 12 * n;
            let o = SCALED_RVA + 12 * n;
            game_write(o, x87::mul(game_read(p), kx));
            game_write(o + 4, x87::mul(game_read(p + 4), ky));
            game_write(o + 8, 0u32);
        }
        for n in 0..POINTS {
            point_turned(addr(SCALED_RVA) + 12 * n);
        }
        if game_read::<u32>(RATE_RVA) != 0 {
            crate::marks::hexagons_frame::hexagons_frame()?;
            let at_now: u32 = game_read(BUFFER_RVA);
            if at_now != HEXAGON_BATCH.records.addr() as u32 {
                let quads = x87::int_over_int_to_int(
                    at_now.wrapping_sub(HEXAGON_BATCH.records.addr() as u32) as i32,
                    game_read(PER_VERTEX_RVA),
                ) as u32;
                game_write(BUFFER_RVA, HEXAGON_BATCH.records.addr() as u32);
                HEXAGON_BATCH.index_count.set(quads.wrapping_mul(12));
                HEXAGON_BATCH.vertex_count.set(quads.wrapping_mul(6));
                crate::marks::hexagons_frame::draw_batch()?;
            }
            let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
            if ((settings + 0x2744) as *const u32).read_unaligned() != 0
                && game_read::<u32>(DRIFTING_RVA) != 0
            {
                crate::marks::hexagons_step::hexagons_step();
            }
        }
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        game_write(COUNT_RVA, 4u32);
    }
    Ok(())
}

use crate::marks::plain::{ld, ldi, st};

/// `frame_axes`'s x87 block as plain Rust on the x87 model (32 instructions), same order and memory places.
unsafe fn frame_axes_x87(mk: usize, s: usize, c: &MarksClock) {
    unsafe {
        let t0 = ld(mk, 0xC932A);
        let t1 = t0 * crate::math::x87_float::int(c.frame_ticks.get() as i32);
        st(mk, 0xB12EA, t1);
        let t2 = ldi(s, 0x2738);
        let t3 = t2 * ld(mk, 0xC932E);
        st(mk, 0xAF7E8, t3);
        let t4 = ldi(s, 0x2720);
        let t5 = ldi(s, 0x2724);
        let t6 = ldi(s, 0x2728);
        st(mk, 0xAFCD6, t6);
        st(mk, 0xAFCD2, t5);
        st(mk, 0xAFCCE, t4);
        let t7 = ldi(s, 0x272C);
        let t8 = t7 - ldi(s, 0x2720);
        st(mk, 0xAFCDA, t8);
        let t9 = ldi(s, 0x2730);
        let t10 = t9 - ldi(s, 0x2724);
        st(mk, 0xAFCDE, t10);
        let t11 = ldi(s, 0x2734);
        let t12 = t11 - ldi(s, 0x2728);
        st(mk, 0xAFCE2, t12);
        let t13 = ld(mk, 0xAFCDA);
        let t14 = t13 * t13;
        let t15 = ld(mk, 0xAFCDE);
        let t16 = t15 * t15;
        let t17 = ld(mk, 0xAFCE2);
        let t18 = t17 * t17;
        let t19 = t16 + t18;
        let t20 = t14 + t19;
        let t21 = t20.sqrt();
        let t22 = t21 / ld(mk, 0xC9332);
        st(mk, 0xAFCC6, t22);
    }
}

/// `frame_up`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
unsafe fn frame_up_x87(mk: usize, s: usize) {
    unsafe {
        let t0 = ldi(s, 0x2708);
        let t1 = t0 - ldi(s, 0x2720);
        st(mk, 0xAFCE6, t1);
        let t2 = ldi(s, 0x270C);
        let t3 = t2 - ldi(s, 0x2724);
        st(mk, 0xAFCEA, t3);
        let t4 = ldi(s, 0x2710);
        let t5 = t4 - ldi(s, 0x2728);
        st(mk, 0xAFCEE, t5);
        let t6 = ld(mk, 0xAFCE6);
        let t7 = t6 * t6;
        let t8 = ld(mk, 0xAFCEA);
        let t9 = t8 * t8;
        let t10 = ld(mk, 0xAFCEE);
        let t11 = t10 * t10;
        let t12 = t9 + t11;
        let t13 = t7 + t12;
        let t14 = t13.sqrt();
        let t15 = t14 / ld(mk, 0xC9336);
        st(mk, 0xAFCCA, t15);
    }
}

/// `frame_place`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
unsafe fn frame_place_x87(mk: usize, s: usize) {
    unsafe {
        let t0 = ldi(s, 0x2720);
        st(mk, 0xB1262, t0);
        let t1 = ldi(s, 0x2724);
        st(mk, 0xB1266, t1);
        let t2 = ldi(s, 0x2728);
        st(mk, 0xB126A, t2);
    }
}

/// `frame_facing`'s x87 block as plain Rust on the x87 model (9 instructions), same order and memory places.
unsafe fn frame_facing_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0xB1262);
        let t1 = t0 * ld(mk, 0xAFCF2);
        let t2 = ld(mk, 0xB1266);
        let t3 = t2 * ld(mk, 0xAFCF6);
        let t4 = ld(mk, 0xB126A);
        let t5 = t4 * ld(mk, 0xAFCFA);
        let t6 = t3 + t5;
        let t7 = t1 + t6;
        st(mk, 0xAF7D8, t7);
    }
}

/// `point_turned`'s x87 block as plain Rust on the x87 model (19 instructions), same order and memory places.
unsafe fn point_turned_x87(mk: usize, p: usize) {
    unsafe {
        let t0 = ld(p, 0);
        let t1 = ld(p, 0x4);
        let t2 = t0 * ld(mk, 0xAFCDA);
        let t3 = t1 * ld(mk, 0xAFCE6);
        let t4 = t2 + t3;
        st(p, 0, t4);
        let t5 = t0 * ld(mk, 0xAFCDE);
        let t6 = t1 * ld(mk, 0xAFCEA);
        let t7 = t5 + t6;
        st(p, 0x4, t7);
        let t8 = t1 * ld(mk, 0xAFCEE);
        let t9 = t0 * ld(mk, 0xAFCE2);
        let t10 = t8 + t9;
        st(p, 0x8, t10);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn frame_axes_x87(mk: usize, s: usize) {
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
            )
        }
    }

    pub(super) unsafe fn frame_up_x87(mk: usize, s: usize) {
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
            )
        }
    }

    pub(super) unsafe fn frame_place_x87(mk: usize, s: usize) {
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

    pub(super) unsafe fn frame_facing_x87(mk: usize) {
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
            )
        }
    }

    pub(super) unsafe fn point_turned_x87(mk: usize, p: usize) {
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
            )
        }
    }
}

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};

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
                    lo: 0x2720,
                    hi: 0x273C,
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
            |b| unsafe { reference::frame_axes_x87(b[0], b[1]) },
            |b| unsafe { super::frame_axes_x87(b[0], b[1], crate::marks::setup::clock_at(b[0])) },
        );
    }

    #[test]
    fn frame_up_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xAFCCA,
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
            |b| unsafe { reference::frame_up_x87(b[0], b[1]) },
            |b| unsafe { super::frame_up_x87(b[0], b[1]) },
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
            |b| unsafe { reference::frame_place_x87(b[0], b[1]) },
            |b| unsafe { super::frame_place_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn frame_facing_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xAF7D8,
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
            |b| unsafe { reference::frame_facing_x87(b[0]) },
            |b| unsafe { super::frame_facing_x87(b[0]) },
        );
    }

    #[test]
    fn point_turned_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xAFCDA,
                    hi: 0xAFCF2,
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
            |b| unsafe { reference::point_turned_x87(b[0], b[1]) },
            |b| unsafe { super::point_turned_x87(b[0], b[1]) },
        );
    }
}
