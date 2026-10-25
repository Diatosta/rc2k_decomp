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
use crate::framework::rva;
use crate::marks::store::{Slot, Where, game_addr, game_read, game_write};
use crate::math::x87;
use crate::race::frame_store::FRAME_WORK;
use crate::race::numbers_store::NUMBERS;
use crate::scene::cross_product3::cross_product3;
use crate::scene::vector3::normalise_vector3;

const AXES: [Slot; 3] = [Slot::at(0x89CF1A), Slot::at(0x89CF26), Slot::at(0x89CF32)];
const PLACE_RVA: Slot = Slot::at(0x8161A2); // flt_8161A2
const FACING_RVA: Slot = Slot::at(0x814718); // flt_814718
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const CORNER_RVA: Slot = Slot::at(0x934C64); // dword_934C64
const MARK_CORNERS_RVA: Slot = Slot::at(0x8EB971); // g_marksCorners
const UV_K_RVA: Slot = Slot::at(0x8EB969); // flt_8EB969
const PLACES_RVA: Slot = Slot::at(0x934F3C); // flt_934F3C .. flt_934F58
const CORNERS_RVA: Slot = Slot::at(0x934C98); // flt_934C98 .. flt_934CC4: four of 12
const KIND_RVA: Slot = Slot::at(0x76529C); // g_marksKindFloat
const LIFTS_RVA: Slot = Slot::at(0x934F5C); // flt_934F5C ..
const RHW_K_RVA: Slot = Slot::at(0x934F6C); // flt_934F6C ..
const Z_K_RVA: Slot = Slot::at(0x8EB96D); // flt_8EB96D
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0xC, 0xE), (8, 0xA), (0, 2), (4, 6)];

fn addr(r: impl Where) -> u32 {
    game_addr(r) as u32
}

/// The original's 0x4C8F6A .. 0x4C9000: the rate (`flt_89CA34`), the place (`flt_89CEF6` ..) and the first axis (`flt_89CF1A` ..) from the settings, its length over `flt_934F34` into `flt_89CEEE`.
unsafe fn plane_axes(s: u32) {
    unsafe { plane_axes_x87(crate::marks::store::base() as usize, s as usize) }
}

/// The original's 0x4C900D .. 0x4C906D: the second axis (`flt_89CF26` ..), its length over `flt_934F38` into `flt_89CEF2`.
unsafe fn plane_up(s: u32) {
    unsafe { plane_up_x87(crate::marks::store::base() as usize, s as usize) }
}

/// The original's 0x4C90BE .. 0x4C90E2: the place as floats into `flt_8161A2` ...
unsafe fn plane_place(s: u32) {
    unsafe { plane_place_x87(crate::marks::store::base() as usize, s as usize) }
}

/// The original's 0x4C90ED .. 0x4C9113: the place's way dotted with the third axis into `flt_814718`.
unsafe fn plane_facing() {
    unsafe { plane_facing_x87(crate::marks::store::base() as usize) }
}

/// The original's 0x4C915A .. 0x4C9244: the plane's centre and corners (`flt_934C98` .. `flt_934CC4`) from the axes.
unsafe fn plane_corners() {
    unsafe { plane_corners_x87(crate::marks::store::base() as usize) }
}

/// The original's 0x4C93FA .. 0x4C94CA: the four corners projected.
unsafe fn plane_projected() {
    unsafe {
        plane_projected_x87(
            crate::race::store::asm_base() as usize,
            crate::marks::store::base() as usize,
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn draw_plane() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        let s: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let corner: u32 = game_read(CORNER_RVA);
        plane_axes(s);
        let [a0, a1, a2] = AXES.map(addr);
        normalise_vector3(a0);
        plane_up(s);
        normalise_vector3(a1);
        cross_product3(a1, a0, a2)?;
        normalise_vector3(a2);
        cross_product3(a0, a2, a1)?;
        crate::marks::scale_axes::scale_axes();
        plane_place(s);
        normalise_vector3(addr(PLACE_RVA));
        plane_facing();
        if game_read::<u32>(FACING_RVA) & 0x8000_0000 == 0 {
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
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        plane_corners();
        let at = addr(MARK_CORNERS_RVA) + corner * 0x80;
        let word = |off: u32| ((at + off) as *const u16).read_unaligned() as u32;
        let uv_k: u32 = game_read(UV_K_RVA);
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex + 0x18, x87::int_times(word(u) as i32, uv_k));
            game_write(vertex + 0x1C, x87::int_times(word(v) as i32, uv_k));
        }
        for n in 0..4 {
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex + 0x18, x87::loaded(game_read(PLACES_RVA + 8 * n)));
            game_write(
                vertex + 0x1C,
                x87::loaded(game_read(PLACES_RVA + 8 * n + 4)),
            );
        }
        let light: u32 = FRAME_WORK.view_light.bits();
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, light);
        }
        for n in 0..4 {
            if game_read::<i32>(CORNERS_RVA + 12 * n + 8) <= (NUMBERS.k4096_a.bits() as i32) {
                return Ok(());
            }
        }
        plane_projected();
        let [kind, z_k]: [u32; 2] = [game_read(KIND_RVA), game_read(Z_K_RVA)];
        for n in 0..4 {
            let c = CORNERS_RVA + 12 * n;
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex, game_read::<u32>(c));
            game_write(vertex + 4, game_read::<u32>(c + 4));
            let depth: u32 = game_read(c + 8);
            game_write(
                vertex + 8,
                x87::sum3_times(depth, kind, game_read(LIFTS_RVA + 4 * n), z_k),
            );
        }
        for n in 0..4 {
            let depth: u32 = game_read(CORNERS_RVA + 12 * n + 8);
            game_write(
                VERTICES_RVA + 0x20 * n + 0xC,
                x87::reciprocal_times(depth, game_read(RHW_K_RVA + 4 * n)),
            );
        }
        crate::marks::clip_quad::draw_fan();
        crate::scene::depth_test_on::depth_test_on();
    }
    Ok(())
}

use crate::marks::plain::{ld, ldi, st};
use crate::math::x87_float as bh;

/// `plane_axes`'s x87 block as plain Rust on the x87 model (29 instructions), same order and memory places.
unsafe fn plane_axes_x87(mk: usize, s: usize) {
    unsafe {
        let t0 = ldi(s, 0x2740);
        let t1 = t0 * ld(mk, 0x1CFFF0);
        st(mk, 0x137AF4, t1);
        let t2 = ldi(s, 0x2720);
        let t3 = ldi(s, 0x2724);
        let t4 = ldi(s, 0x2728);
        st(mk, 0x137FBE, t4);
        st(mk, 0x137FBA, t3);
        st(mk, 0x137FB6, t2);
        let t5 = ldi(s, 0x272C);
        let t6 = t5 - ldi(s, 0x2720);
        st(mk, 0x137FDA, t6);
        let t7 = ldi(s, 0x2730);
        let t8 = t7 - ldi(s, 0x2724);
        st(mk, 0x137FDE, t8);
        let t9 = ldi(s, 0x2734);
        let t10 = t9 - ldi(s, 0x2728);
        st(mk, 0x137FE2, t10);
        let t11 = ld(mk, 0x137FDA);
        let t12 = t11 * t11;
        let t13 = ld(mk, 0x137FDE);
        let t14 = t13 * t13;
        let t15 = ld(mk, 0x137FE2);
        let t16 = t15 * t15;
        let t17 = t14 + t16;
        let t18 = t12 + t17;
        let t19 = t18.sqrt();
        let t20 = t19 / ld(mk, 0x1CFFF4);
        st(mk, 0x137FAE, t20);
    }
}

/// `plane_up`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
unsafe fn plane_up_x87(mk: usize, s: usize) {
    unsafe {
        let t0 = ldi(s, 0x2708);
        let t1 = t0 - ldi(s, 0x2720);
        st(mk, 0x137FE6, t1);
        let t2 = ldi(s, 0x270C);
        let t3 = t2 - ldi(s, 0x2724);
        st(mk, 0x137FEA, t3);
        let t4 = ldi(s, 0x2710);
        let t5 = t4 - ldi(s, 0x2728);
        st(mk, 0x137FEE, t5);
        let t6 = ld(mk, 0x137FE6);
        let t7 = t6 * t6;
        let t8 = ld(mk, 0x137FEA);
        let t9 = t8 * t8;
        let t10 = ld(mk, 0x137FEE);
        let t11 = t10 * t10;
        let t12 = t9 + t11;
        let t13 = t7 + t12;
        let t14 = t13.sqrt();
        let t15 = t14 / ld(mk, 0x1CFFF8);
        st(mk, 0x137FB2, t15);
    }
}

/// `plane_place`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
unsafe fn plane_place_x87(mk: usize, s: usize) {
    unsafe {
        let t0 = ldi(s, 0x2720);
        st(mk, 0xB1262, t0);
        let t1 = ldi(s, 0x2724);
        st(mk, 0xB1266, t1);
        let t2 = ldi(s, 0x2728);
        st(mk, 0xB126A, t2);
    }
}

/// `plane_facing`'s x87 block as plain Rust on the x87 model (9 instructions), same order and memory places.
unsafe fn plane_facing_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0xB1262);
        let t1 = t0 * ld(mk, 0x137FF2);
        let t2 = ld(mk, 0xB1266);
        let t3 = t2 * ld(mk, 0x137FF6);
        let t4 = ld(mk, 0xB126A);
        let t5 = t4 * ld(mk, 0x137FFA);
        let t6 = t3 + t5;
        let t7 = t1 + t6;
        st(mk, 0xAF7D8, t7);
    }
}

/// `plane_corners`'s x87 block as plain Rust on the x87 model (47 instructions), same order and memory places.
unsafe fn plane_corners_x87(mk: usize) {
    unsafe {
        let t0 = bh::X(0.0);
        let t1 = bh::X(0.0);
        let t2 = t1 * ld(mk, 0x137FB2);
        let t3 = t0 * ld(mk, 0x137FAE);
        let t4 = t3 * ld(mk, 0x137FDA);
        let t5 = t2 * ld(mk, 0x137FE6);
        let t6 = t4 + t5;
        let t7 = t6 + ld(mk, 0x137FB6);
        st(mk, 0x1CFD7C, t7);
        let t8 = t3 * ld(mk, 0x137FDE);
        let t9 = t2 * ld(mk, 0x137FEA);
        let t10 = t8 + t9;
        let t11 = t10 + ld(mk, 0x137FBA);
        st(mk, 0x1CFD80, t11);
        let t12 = t2 * ld(mk, 0x137FEE);
        let t13 = t3 * ld(mk, 0x137FE2);
        let t14 = t12 + t13;
        let t15 = t14 + ld(mk, 0x137FBE);
        st(mk, 0x1CFD84, t15);
        let t16 = ld(mk, 0x1CFD7C);
        let t17 = t16 + ld(mk, 0x13803A);
        st(mk, 0x1CFD58, t17);
        let t18 = t17 + ld(mk, 0x13802E);
        st(mk, 0x1CFD64, t18);
        let t19 = t18 - ld(mk, 0x13803A);
        st(mk, 0x1CFD70, t19);
        let t20 = ld(mk, 0x1CFD80);
        let t21 = t20 + ld(mk, 0x13803E);
        st(mk, 0x1CFD5C, t21);
        let t22 = t21 + ld(mk, 0x138032);
        st(mk, 0x1CFD68, t22);
        let t23 = t22 - ld(mk, 0x13803E);
        st(mk, 0x1CFD74, t23);
        let t24 = ld(mk, 0x1CFD84);
        let t25 = t24 + ld(mk, 0x138042);
        st(mk, 0x1CFD60, t25);
        let t26 = t25 + ld(mk, 0x138036);
        st(mk, 0x1CFD6C, t26);
        let t27 = t26 - ld(mk, 0x138042);
        st(mk, 0x1CFD78, t27);
    }
}

/// `plane_projected`'s x87 block as plain Rust on the x87 model (40 instructions), same order and memory places.
unsafe fn plane_projected_x87(rc: usize, mk: usize) {
    unsafe {
        let t0 = ld(rc, 0x224088);
        let t1 = t0 / ld(mk, 0x1CFD60);
        let t2 = t1 * ld(mk, 0x1CFD58);
        let t3 = t2 + ld(rc, 0x22409C);
        st(mk, 0x1CFD58, t3);
        let t4 = t1 * ld(mk, 0x1CFD5C);
        let t5 = -t4;
        let t6 = t5 + ld(rc, 0x2240A0);
        st(mk, 0x1CFD5C, t6);
        let t7 = ld(rc, 0x224088);
        let t8 = t7 / ld(mk, 0x1CFD6C);
        let t9 = t8 * ld(mk, 0x1CFD64);
        let t10 = t9 + ld(rc, 0x22409C);
        st(mk, 0x1CFD64, t10);
        let t11 = t8 * ld(mk, 0x1CFD68);
        let t12 = -t11;
        let t13 = t12 + ld(rc, 0x2240A0);
        st(mk, 0x1CFD68, t13);
        let t14 = ld(rc, 0x224088);
        let t15 = t14 / ld(mk, 0x1CFD78);
        let t16 = t15 * ld(mk, 0x1CFD70);
        let t17 = t16 + ld(rc, 0x22409C);
        st(mk, 0x1CFD70, t17);
        let t18 = t15 * ld(mk, 0x1CFD74);
        let t19 = -t18;
        let t20 = t19 + ld(rc, 0x2240A0);
        st(mk, 0x1CFD74, t20);
        let t21 = ld(rc, 0x224088);
        let t22 = t21 / ld(mk, 0x1CFD84);
        let t23 = t22 * ld(mk, 0x1CFD7C);
        let t24 = t23 + ld(rc, 0x22409C);
        st(mk, 0x1CFD7C, t24);
        let t25 = t22 * ld(mk, 0x1CFD80);
        let t26 = -t25;
        let t27 = t26 + ld(rc, 0x2240A0);
        st(mk, 0x1CFD80, t27);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn plane_axes_x87(mk: usize, s: usize) {
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

    pub(super) unsafe fn plane_up_x87(mk: usize, s: usize) {
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

    pub(super) unsafe fn plane_place_x87(mk: usize, s: usize) {
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

    pub(super) unsafe fn plane_facing_x87(mk: usize) {
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

    pub(super) unsafe fn plane_corners_x87(mk: usize) {
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

    pub(super) unsafe fn plane_projected_x87(rc: usize, mk: usize) {
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

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};

    #[test]
    fn plane_axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137AF4,
                    hi: 0x1CFFF8,
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
            |b| unsafe { reference::plane_axes_x87(b[0], b[1]) },
            |b| unsafe { super::plane_axes_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn plane_up_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137FB2,
                    hi: 0x1CFFFC,
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
            |b| unsafe { reference::plane_up_x87(b[0], b[1]) },
            |b| unsafe { super::plane_up_x87(b[0], b[1]) },
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
            |b| unsafe { reference::plane_place_x87(b[0], b[1]) },
            |b| unsafe { super::plane_place_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn plane_facing_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xAF7D8,
                hi: 0x137FFE,
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
            |b| unsafe { reference::plane_facing_x87(b[0]) },
            |b| unsafe { super::plane_facing_x87(b[0]) },
        );
    }

    #[test]
    fn plane_corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x137FAE,
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
            |b| unsafe { reference::plane_corners_x87(b[0]) },
            |b| unsafe { super::plane_corners_x87(b[0]) },
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
                    lo: 0x1CFD58,
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
            |b| unsafe { reference::plane_projected_x87(b[0], b[1]) },
            |b| unsafe { super::plane_projected_x87(b[0], b[1]) },
        );
    }
}
