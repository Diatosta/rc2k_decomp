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
//! Caller: `sub_4C74DA` (0x4C752F). The x87 runs are the original's instructions over the
//! game's variables; the vector helpers and `scale_axes` their Rust versions. Refuses null
//! settings and a corner past `g_marksCorners`, before anything.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::setup::SETTINGS_RVA;
use crate::marks::store::{Slot, Where, game_addr, game_read, game_write};
use crate::math::x87;
use crate::scene::cross_product3::cross_product3;
use crate::scene::vector3::normalise_vector3;

const AXES: [Slot; 3] = [Slot::at(0x89CF1A), Slot::at(0x89CF26), Slot::at(0x89CF32)];
const PLACE_RVA: Slot = Slot::at(0x8161A2); // flt_8161A2
const FACING_RVA: Slot = Slot::at(0x814718); // flt_814718
const SWITCH_RVA: usize = rva(0x641678); // dword_641678
const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const CORNER_RVA: Slot = Slot::at(0x934C64); // dword_934C64
const MARK_CORNERS_RVA: Slot = Slot::at(0x8EB971); // g_marksCorners
const UV_K_RVA: Slot = Slot::at(0x8EB969); // flt_8EB969
const PLACES_RVA: Slot = Slot::at(0x934F3C); // flt_934F3C .. flt_934F58
const LIGHT_RVA: usize = rva(0x6415E4); // g_viewLight
const CORNERS_RVA: Slot = Slot::at(0x934C98); // flt_934C98 .. flt_934CC4: four of 12
const NEAR_RVA: usize = rva(0x5C5E18); // flt_5C5E18
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
            mk = in(reg) crate::marks::store::base(),
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C900D .. 0x4C906D: the second axis (`flt_89CF26` ..), its length over `flt_934F38` into `flt_89CEF2`.
unsafe fn plane_up(s: u32) {
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
            mk = in(reg) crate::marks::store::base(),
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C90BE .. 0x4C90E2: the place as floats into `flt_8161A2` ...
unsafe fn plane_place(s: u32) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0xB1262]",
            "fild dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0xB1266]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0xB126A]",
            mk = in(reg) crate::marks::store::base(),
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C90ED .. 0x4C9113: the place's way dotted with the third axis into `flt_814718`.
unsafe fn plane_facing() {
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
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C915A .. 0x4C9244: the plane's centre and corners (`flt_934C98` .. `flt_934CC4`) from the axes.
unsafe fn plane_corners() {
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
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4C93FA .. 0x4C94CA: the four corners projected.
unsafe fn plane_projected() {
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
            rc = in(reg) crate::race::store::asm_base(),
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn draw_plane() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
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
        if game_read::<u32>(SWITCH_RVA) == 0 {
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
        let light: u32 = game_read(LIGHT_RVA);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, light);
        }
        for n in 0..4 {
            if game_read::<i32>(CORNERS_RVA + 12 * n + 8) <= game_read::<i32>(NEAR_RVA) {
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
