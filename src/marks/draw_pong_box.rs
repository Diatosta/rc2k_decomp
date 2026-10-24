//! `marks::draw_pong_box` (0x4B8599): one box of the pong game (ball, bat or a score
//! cell) drawn on the squares' frame: its height (st(0)) along the second axis times
//! `flt_89CEF2` into `flt_768664` .. (and times `flt_765158` into `flt_76867C` ..), its
//! width (st(1)) along the first times `flt_89CEEE` into `flt_768658` .. (`flt_768670` ..);
//! the texture places from `dword_76920A` ..; its place (st(2) along, st(3) across) on the
//! frame into `flt_89CF92` .., the four corners; unless the first corner's depth is below
//! `flt_76921A`: projected, depth times `flt_89CEEA`, colour -1, `scene::draw_fan`.
//!
//! In:  st(0) height, st(1) width, st(2) along, st(3) across (popped)
//! Out: nothing (its caller reads no register after it)
//!
//! Callers: `marks::run_pong` (ball, bat), `marks::pong_cell`. The x87 runs are the
//! original's instructions over the game's variables; the draw through `part` (1
//! `draw_fan`), in the game its Rust version. Thunk by `four_in_hook!`. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose draw goes to a stand-in,
//! against the Rust version, boxes near and far.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::rva;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: Slot = Slot::at(0x76920A); // dword_76920A .. dword_769216
// flt_768658 .. flt_768684
const CORNERS_RVA: Slot = Slot::at(0x89CF92); // flt_89CF92 .. flt_89CFBE
const NEAR_RVA: Slot = Slot::at(0x76921A); // flt_76921A

/// The original's 0x4B8599 .. 0x4B8659 on the four values: the sides; the place's two
/// values (left untouched) popped after.
unsafe fn sides(v: &[[u8; 10]; 4]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{v} + 30]",
            "fld tbyte ptr [{v} + 20]",
            "fld tbyte ptr [{v} + 10]",
            "fld tbyte ptr [{v}]",
            "fld dword ptr [{mk} + 0x137FE6]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fst dword ptr [{mk} + 0x3724]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x373C]",
            "fld dword ptr [{mk} + 0x137FEA]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fst dword ptr [{mk} + 0x3728]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3740]",
            "fld dword ptr [{mk} + 0x137FEE]",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fst dword ptr [{mk} + 0x372C]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3744]",
            "fld dword ptr [{mk} + 0x137FDA]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fst dword ptr [{mk} + 0x3718]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3730]",
            "fld dword ptr [{mk} + 0x137FDE]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fst dword ptr [{mk} + 0x371C]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3734]",
            "fld dword ptr [{mk} + 0x137FE2]",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fst dword ptr [{mk} + 0x3720]",
            "fmul dword ptr [{mk} + 0x218]",
            "fstp dword ptr [{mk} + 0x3738]",
            "fstp st(0)",
            "fstp st(0)",
            mk = in(reg) crate::marks::store::base(),
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4B868E .. 0x4B8786 on the place: the corners.
unsafe fn corners(v: &[[u8; 10]; 4]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{v} + 30]",
            "fld tbyte ptr [{v} + 20]",
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
            "fstp dword ptr [{mk} + 0x138052]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FDE]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x137FEA]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FBA]",
            "fstp dword ptr [{mk} + 0x138056]",
            "fmul dword ptr [{mk} + 0x137FEE]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x137FE2]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{mk} + 0x137FBE]",
            "fstp dword ptr [{mk} + 0x13805A]",
            "fld dword ptr [{mk} + 0x138052]",
            "fst dword ptr [{mk} + 0x138052]",
            "fadd dword ptr [{mk} + 0x3724]",
            "fst dword ptr [{mk} + 0x13805E]",
            "fadd dword ptr [{mk} + 0x3718]",
            "fst dword ptr [{mk} + 0x13806A]",
            "fsub dword ptr [{mk} + 0x3724]",
            "fstp dword ptr [{mk} + 0x138076]",
            "fld dword ptr [{mk} + 0x138056]",
            "fst dword ptr [{mk} + 0x138056]",
            "fadd dword ptr [{mk} + 0x3728]",
            "fst dword ptr [{mk} + 0x138062]",
            "fadd dword ptr [{mk} + 0x371C]",
            "fst dword ptr [{mk} + 0x13806E]",
            "fsub dword ptr [{mk} + 0x3728]",
            "fstp dword ptr [{mk} + 0x13807A]",
            "fld dword ptr [{mk} + 0x13805A]",
            "fst dword ptr [{mk} + 0x13805A]",
            "fadd dword ptr [{mk} + 0x372C]",
            "fst dword ptr [{mk} + 0x138066]",
            "fadd dword ptr [{mk} + 0x3720]",
            "fst dword ptr [{mk} + 0x138072]",
            "fsub dword ptr [{mk} + 0x372C]",
            "fstp dword ptr [{mk} + 0x13807E]",
            mk = in(reg) crate::marks::store::base(),
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4B87A5 .. 0x4B88A9: the corners projected, their depth.
unsafe fn projected(p: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x13807E]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x138072]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x138066]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x13805A]",
            "fld dword ptr [{mk} + 0x138056]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x138052]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x4]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{p} + 0x0]",
            "fld dword ptr [{mk} + 0x138062]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x13805E]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x24]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{p} + 0x20]",
            "fld dword ptr [{mk} + 0x13806E]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x13806A]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x44]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{p} + 0x40]",
            "fld dword ptr [{mk} + 0x13807A]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x138076]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x64]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{p} + 0x60]",
            "fld dword ptr [{mk} + 0x13805A]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x138066]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x138072]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fld dword ptr [{mk} + 0x13807E]",
            "fmul dword ptr [{mk} + 0x137FAA]",
            "fstp dword ptr [{p} + 0x68]",
            "fstp dword ptr [{p} + 0x48]",
            "fstp dword ptr [{p} + 0x28]",
            "fstp dword ptr [{p} + 0x8]",
            rc = in(reg) crate::race::store::asm_base(),
            mk = in(reg) crate::marks::store::base(),
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header).
pub unsafe fn draw_pong_box(v: [[u8; 10]; 4]) {
    unsafe {
        sides(&v);
        let [pa, pb, pc, pd]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, w)) in [(pa, pb), (pc, pb), (pc, pd), (pa, pd)]
            .into_iter()
            .enumerate()
        {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, w);
        }
        corners(&v);
        if x87::compare_status(game_read(CORNERS_RVA + 8), game_read(NEAR_RVA)) == 0x100 {
            return;
        }
        projected(game_addr(VERTICES_RVA) as u32);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
        }
        crate::marks::clip_quad::draw_fan();
    }
}
