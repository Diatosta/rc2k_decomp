//! `marks::draw_square` (0x4BF80F): one square item drawn on the squares' frame. The four
//! vertices' texture places from `dword_8BC1AE` .. (u a, c, c, a; v b, b, d, d) plus the
//! item's +0xC / +0x10; its place (st(1) the across share times `flt_89CEEE`, st(0) the
//! along share times `flt_89CEF2`) along the two axes from the frame's place into
//! `flt_89CF92` ..; less `flt_89CF56` .. and `flt_89CF62` ..; the four corners (plus
//! `flt_89CF4A` .., `flt_89CF3E` .., less `flt_89CF4A` ..) into `flt_89CF92` .. `flt_89CFBE`;
//! unless the first corner's depth is at or below 4096.0 (a signed compare of the bits):
//! projected (`flt_624088` over each depth, the centre `flt_62409C` / `flt_6240A0`), depth
//! (plus `g_marksKindFloat`, times `flt_89CEEA`), colour edx, `scene::draw_fan`.
//!
//! In:  st(0) the along share, st(1) the across share (popped), esi the item, edx the
//!      colour
//! Out: eax, ebx, ecx the places a, b, c; ebp `g_polygonVertices`; the rest as `draw_fan`
//!      leaves them when drawn
//!
//! Caller: `sub_4BFFEC` (0x4C00FE). The x87 runs are the original's instructions over the
//! game's variables; the draw through `part` (1 `draw_fan`), in the game its Rust version.
//! Thunk by `two_in_hook!`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose draw goes to a stand-in, against the Rust version, items near and far.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: usize = rva(0x8BC1AE); // dword_8BC1AE .. dword_8BC1BA
const CORNERS_RVA: usize = rva(0x89CF92); // flt_89CF92 .. flt_89CFBE: four of 12
const FAR_ENOUGH: i32 = 0x4580_0000; // 4096.0

/// A call by place: 1 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The original's 0x4BF845 .. 0x4BF88D: the item's texture shift added to the places.
unsafe fn shifted_places(s: u32, p: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x18]",
            "fstp dword ptr [{p} + 0x18]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x38]",
            "fstp dword ptr [{p} + 0x38]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x58]",
            "fstp dword ptr [{p} + 0x58]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x78]",
            "fstp dword ptr [{p} + 0x78]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x1C]",
            "fstp dword ptr [{p} + 0x1C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x3C]",
            "fstp dword ptr [{p} + 0x3C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x5C]",
            "fstp dword ptr [{p} + 0x5C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x7C]",
            "fstp dword ptr [{p} + 0x7C]",
            s = in(reg) s,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF88E .. 0x4BF9AA on the two values: the corners.
unsafe fn corners(b: u32, v: &[[u8; 10]; 2]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{v} + 10]",
            "fld tbyte ptr [{v}]",
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
            "fsub dword ptr [{b} + 0x49CF56]",
            "fsub dword ptr [{b} + 0x49CF62]",
            "fst dword ptr [{b} + 0x49CF92]",
            "fadd dword ptr [{b} + 0x49CF4A]",
            "fst dword ptr [{b} + 0x49CF9E]",
            "fadd dword ptr [{b} + 0x49CF3E]",
            "fst dword ptr [{b} + 0x49CFAA]",
            "fsub dword ptr [{b} + 0x49CF4A]",
            "fstp dword ptr [{b} + 0x49CFB6]",
            "fld dword ptr [{b} + 0x49CF96]",
            "fsub dword ptr [{b} + 0x49CF5A]",
            "fsub dword ptr [{b} + 0x49CF66]",
            "fst dword ptr [{b} + 0x49CF96]",
            "fadd dword ptr [{b} + 0x49CF4E]",
            "fst dword ptr [{b} + 0x49CFA2]",
            "fadd dword ptr [{b} + 0x49CF42]",
            "fst dword ptr [{b} + 0x49CFAE]",
            "fsub dword ptr [{b} + 0x49CF4E]",
            "fstp dword ptr [{b} + 0x49CFBA]",
            "fld dword ptr [{b} + 0x49CF9A]",
            "fsub dword ptr [{b} + 0x49CF5E]",
            "fsub dword ptr [{b} + 0x49CF6A]",
            "fst dword ptr [{b} + 0x49CF9A]",
            "fadd dword ptr [{b} + 0x49CF52]",
            "fst dword ptr [{b} + 0x49CFA6]",
            "fadd dword ptr [{b} + 0x49CF46]",
            "fst dword ptr [{b} + 0x49CFB2]",
            "fsub dword ptr [{b} + 0x49CF52]",
            "fstp dword ptr [{b} + 0x49CFBE]",
            b = in(reg) b,
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF9BA .. 0x4BFAD6: the corners projected, their depth.
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
            "fadd dword ptr [{b} + 0x36529C]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFA6]",
            "fadd dword ptr [{b} + 0x36529C]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFB2]",
            "fadd dword ptr [{b} + 0x36529C]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFBE]",
            "fadd dword ptr [{b} + 0x36529C]",
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
pub unsafe fn draw_square(
    entry: &Regs,
    shares: [[u8; 10]; 2],
    part: Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let b = game_addr(0) as u32;
        g.ebp = game_addr(VERTICES_RVA) as u32;
        let [a, pb, c, d]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, v)) in [(a, pb), (c, pb), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        [g.eax, g.ebx, g.ecx] = [a, pb, c];
        shifted_places(g.esi, g.ebp);
        corners(b, &shares);
        if game_read::<i32>(CORNERS_RVA + 8) <= FAR_ENOUGH {
            return Ok(g);
        }
        projected(b, g.ebp);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, g.edx);
        }
        part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `draw_fan`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part) };
    Ok(())
}
