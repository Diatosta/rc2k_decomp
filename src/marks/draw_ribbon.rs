//! `marks::draw_ribbon` (0x4C5E63): a ribbon piece between two places already in the
//! view's frame (`flt_8EB8D5` .. and `dword_8EB8E1` ..): half the width (edx >> 1, whole)
//! in `dword_8EB8B5`, the four vertices' colour (ecx << 24) | esi and texture places from
//! cell eax of `g_marksCellPlaces`; unless either depth is at or below `dword_8EB961` (a
//! signed compare of the bits): both projected (`flt_8EB8CD`, `flt_8EB8D1` their scales),
//! the way between them made one long and turned a quarter (`flt_8F3985`, `flt_8F3989`),
//! the sides that times the half width and each scale (`flt_8F3991` ..), the four corners,
//! the depths times `flt_8EB96D`; `marks::clip_quad`, and unless all is outside
//! `scene::draw_fan` and `g_polygonVertexCount` 4.
//!
//! In:  eax the cell, ecx the alpha, edx the width, esi the colour (and the two places)
//! Out: nothing (its caller reads no register after it)
//!
//! Callers: the marks' lines (0x4C5D65) and 0x4CFAA3. The x87 block is the original's
//! instructions over the game's variables. Refuses, before anything, a cell past 0x40 and
//! null marks settings (the clip's refusal).

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::clip_quad::{clip_quad, draw_fan};
use crate::marks::store::{Slot, game_addr, game_read, game_write};

const HALF_RVA: Slot = Slot::at(0x8EB8B5); // dword_8EB8B5
const DEPTHS_RVA: [Slot; 2] = [Slot::at(0x8EB8DD), Slot::at(0x8EB8E9)];
const NEAR_RVA: Slot = Slot::at(0x8EB961); // dword_8EB961
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const CELLS_RVA: Slot = Slot::at(0x935766); // g_marksCellPlaces: 0x41 of 16 bytes
const CELLS: u32 = 0x41;

/// The original's 0x4C5EF7 .. 0x4C60E9: the two projected and the ribbon's corners.
unsafe fn ribbon() {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x18699D]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x186995]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x186999]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x186999]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x186995]",
            "fstp dword ptr [{mk} + 0x18698D]",
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x1869A9]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x1869A1]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x1869A5]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{mk} + 0x1869A5]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{mk} + 0x1869A1]",
            "fstp dword ptr [{mk} + 0x186991]",
            "fld dword ptr [{mk} + 0x1869A1]",
            "fsub dword ptr [{mk} + 0x186995]",
            "fstp dword ptr [{mk} + 0x18EA45]",
            "fld dword ptr [{mk} + 0x1869A5]",
            "fsub dword ptr [{mk} + 0x186999]",
            "fstp dword ptr [{mk} + 0x18EA49]",
            "fld dword ptr [{mk} + 0x18EA45]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x18EA49]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            ".byte 0xD9, 0xE8",
            ".byte 0xDE, 0xF1",
            "fld dword ptr [{mk} + 0x18EA45]",
            "fld dword ptr [{mk} + 0x18EA49]",
            ".byte 0xD8, 0xCA",
            "fstp dword ptr [{mk} + 0x18EA45]",
            ".byte 0xDE, 0xC9",
            ".byte 0xD9, 0xE0",
            "fstp dword ptr [{mk} + 0x18EA49]",
            "fld dword ptr [{mk} + 0x18EA45]",
            "fimul dword ptr [{mk} + 0x186975]",
            "fmul dword ptr [{mk} + 0x18698D]",
            "fstp dword ptr [{mk} + 0x18EA51]",
            "fld dword ptr [{mk} + 0x18EA49]",
            "fimul dword ptr [{mk} + 0x186975]",
            "fmul dword ptr [{mk} + 0x18698D]",
            "fstp dword ptr [{mk} + 0x18EA55]",
            "fld dword ptr [{mk} + 0x18EA45]",
            "fimul dword ptr [{mk} + 0x186975]",
            "fmul dword ptr [{mk} + 0x186991]",
            "fstp dword ptr [{mk} + 0x18EA5D]",
            "fld dword ptr [{mk} + 0x18EA49]",
            "fimul dword ptr [{mk} + 0x186975]",
            "fmul dword ptr [{mk} + 0x186991]",
            "fstp dword ptr [{mk} + 0x18EA61]",
            "fld dword ptr [{mk} + 0x186995]",
            "fsub dword ptr [{mk} + 0x18EA51]",
            "fstp dword ptr [{bf} + 0x5605A0]",
            "fld dword ptr [{mk} + 0x186999]",
            "fsub dword ptr [{mk} + 0x18EA55]",
            "fstp dword ptr [{bf} + 0x5605A4]",
            "fld dword ptr [{mk} + 0x186995]",
            "fadd dword ptr [{mk} + 0x18EA51]",
            "fstp dword ptr [{bf} + 0x5605C0]",
            "fld dword ptr [{mk} + 0x186999]",
            "fadd dword ptr [{mk} + 0x18EA55]",
            "fstp dword ptr [{bf} + 0x5605C4]",
            "fld dword ptr [{mk} + 0x1869A1]",
            "fadd dword ptr [{mk} + 0x18EA5D]",
            "fstp dword ptr [{bf} + 0x5605E0]",
            "fld dword ptr [{mk} + 0x1869A5]",
            "fadd dword ptr [{mk} + 0x18EA61]",
            "fstp dword ptr [{bf} + 0x5605E4]",
            "fld dword ptr [{mk} + 0x1869A1]",
            "fsub dword ptr [{mk} + 0x18EA5D]",
            "fstp dword ptr [{bf} + 0x560600]",
            "fld dword ptr [{mk} + 0x1869A5]",
            "fsub dword ptr [{mk} + 0x18EA61]",
            "fstp dword ptr [{bf} + 0x560604]",
            "fld dword ptr [{mk} + 0x18699D]",
            "fmul dword ptr [{mk} + 0x186A2D]",
            "fst dword ptr [{bf} + 0x5605A8]",
            "fstp dword ptr [{bf} + 0x5605C8]",
            "fld dword ptr [{mk} + 0x1869A9]",
            "fmul dword ptr [{mk} + 0x186A2D]",
            "fst dword ptr [{bf} + 0x5605E8]",
            "fstp dword ptr [{bf} + 0x560608]",
            rc = in(reg) crate::race::store::asm_base(),
            bf = in(reg) crate::race::store::buffers_asm_base(),
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): `cell` (eax), `alpha` (ecx), `width` (edx),
/// `colour` (esi). Its one caller reads no register after it.
pub unsafe fn draw_ribbon(cell: u32, alpha: u32, width: u32, colour: u32) -> Result<(), Refused> {
    if cell >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    unsafe {
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        game_write(HALF_RVA, width >> 1);
        let colour = (alpha << 24) | colour;
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, colour);
        }
        let at = cell << 4;
        let place = |n: usize| game_read::<u32>(CELLS_RVA + at as usize + 4 * n);
        let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
        for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
            .into_iter()
            .enumerate()
        {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        for r in DEPTHS_RVA {
            if game_read::<i32>(r) <= game_read::<i32>(NEAR_RVA) {
                return Ok(());
            }
        }
        ribbon();
        if clip_quad(game_addr(VERTICES_RVA) as u32)? {
            return Ok(());
        }
        draw_fan();
        game_write(VERTEX_COUNT_RVA, 4u32);
    }
    Ok(())
}
