//! `marks::draw_pong_box` (0x4B8599): one box of the pong game (ball, bat or a score
//! cell) drawn on the squares' frame: its height (st(0)) along the second axis times
//! `flt_89CEF2` into `flt_768664` .. (and times `flt_765158` into `flt_76867C` ..), its
//! width (st(1)) along the first times `flt_89CEEE` into `flt_768658` .. (`flt_768670` ..);
//! the texture places from `dword_76920A` ..; its place (st(2) along, st(3) across) on the
//! frame into `flt_89CF92` .., the four corners; unless the first corner's depth is below
//! `flt_76921A`: projected, depth times `flt_89CEEA`, colour -1, `scene::draw_fan`.
//!
//! In:  st(0) height, st(1) width, st(2) along, st(3) across (popped)
//! Out: eax the depth compare's status (& 0x700), ebx .. edx the places b .. d, ebp
//!      `g_polygonVertices`; the rest as `draw_fan` leaves them when drawn
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

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: usize = rva(0x76920A); // dword_76920A .. dword_769216
// flt_768658 .. flt_768684
const CORNERS_RVA: usize = rva(0x89CF92); // flt_89CF92 .. flt_89CFBE
const NEAR_RVA: usize = rva(0x76921A); // flt_76921A

/// The original's 0x4B8599 .. 0x4B8659 on the four values: the sides; the place's two
/// values (left untouched) popped after.
unsafe fn sides(b: u32, v: &[[u8; 10]; 4]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{v} + 30]",
            "fld tbyte ptr [{v} + 20]",
            "fld tbyte ptr [{v} + 10]",
            "fld tbyte ptr [{v}]",
            "fld dword ptr [{b} + 0x49CF26]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fst dword ptr [{b} + 0x368664]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x36867C]",
            "fld dword ptr [{b} + 0x49CF2A]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fst dword ptr [{b} + 0x368668]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368680]",
            "fld dword ptr [{b} + 0x49CF2E]",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fst dword ptr [{b} + 0x36866C]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368684]",
            "fld dword ptr [{b} + 0x49CF1A]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fst dword ptr [{b} + 0x368658]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368670]",
            "fld dword ptr [{b} + 0x49CF1E]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fst dword ptr [{b} + 0x36865C]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368674]",
            "fld dword ptr [{b} + 0x49CF22]",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fst dword ptr [{b} + 0x368660]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368678]",
            "fstp st(0)",
            "fstp st(0)",
            b = in(reg) b,
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4B868E .. 0x4B8786 on the place: the corners.
unsafe fn corners(b: u32, v: &[[u8; 10]; 4]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{v} + 30]",
            "fld tbyte ptr [{v} + 20]",
            "fmul dword ptr [{b} + 0x49CEF2]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CEEE]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF1A]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF26]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x49CEF6]",
            "fstp dword ptr [{b} + 0x49CF92]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF1E]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF2A]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x49CEFA]",
            "fstp dword ptr [{b} + 0x49CF96]",
            "fmul dword ptr [{b} + 0x49CF2E]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CF22]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x49CEFE]",
            "fstp dword ptr [{b} + 0x49CF9A]",
            "fld dword ptr [{b} + 0x49CF92]",
            "fst dword ptr [{b} + 0x49CF92]",
            "fadd dword ptr [{b} + 0x368664]",
            "fst dword ptr [{b} + 0x49CF9E]",
            "fadd dword ptr [{b} + 0x368658]",
            "fst dword ptr [{b} + 0x49CFAA]",
            "fsub dword ptr [{b} + 0x368664]",
            "fstp dword ptr [{b} + 0x49CFB6]",
            "fld dword ptr [{b} + 0x49CF96]",
            "fst dword ptr [{b} + 0x49CF96]",
            "fadd dword ptr [{b} + 0x368668]",
            "fst dword ptr [{b} + 0x49CFA2]",
            "fadd dword ptr [{b} + 0x36865C]",
            "fst dword ptr [{b} + 0x49CFAE]",
            "fsub dword ptr [{b} + 0x368668]",
            "fstp dword ptr [{b} + 0x49CFBA]",
            "fld dword ptr [{b} + 0x49CF9A]",
            "fst dword ptr [{b} + 0x49CF9A]",
            "fadd dword ptr [{b} + 0x36866C]",
            "fst dword ptr [{b} + 0x49CFA6]",
            "fadd dword ptr [{b} + 0x368660]",
            "fst dword ptr [{b} + 0x49CFB2]",
            "fsub dword ptr [{b} + 0x36866C]",
            "fstp dword ptr [{b} + 0x49CFBE]",
            b = in(reg) b,
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4B87A5 .. 0x4B88A9: the corners projected, their depth.
unsafe fn projected(b: u32, p: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x49CFBE]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x49CFB2]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x49CFA6]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x49CF9A]",
            "fld dword ptr [{b} + 0x49CF96]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CF92]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x4]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x0]",
            "fld dword ptr [{b} + 0x49CFA2]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CF9E]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x24]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x20]",
            "fld dword ptr [{b} + 0x49CFAE]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CFAA]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x44]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x40]",
            "fld dword ptr [{b} + 0x49CFBA]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CFB6]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x64]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x60]",
            "fld dword ptr [{b} + 0x49CF9A]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFA6]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFB2]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFBE]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fstp dword ptr [{p} + 0x68]",
            "fstp dword ptr [{p} + 0x48]",
            "fstp dword ptr [{p} + 0x28]",
            "fstp dword ptr [{p} + 0x8]",
            b = in(reg) b,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_pong_box(entry: &Regs, v: [[u8; 10]; 4]) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let b = game_addr(0) as u32;
        sides(b, &v);
        g.ebp = game_addr(VERTICES_RVA) as u32;
        let [pa, pb, pc, pd]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, w)) in [(pa, pb), (pc, pb), (pc, pd), (pa, pd)]
            .into_iter()
            .enumerate()
        {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, w);
        }
        [g.eax, g.ebx, g.ecx, g.edx] = [pa, pb, pc, pd];
        corners(b, &v);
        g.eax = x87::compare_status(game_read(CORNERS_RVA + 8), game_read(NEAR_RVA)) as u32;
        if g.eax == 0x100 {
            return Ok(g);
        }
        projected(b, g.ebp);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
        }
        crate::marks::draw_pong_box::game_part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `draw_fan`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part) };
    Ok(())
}
