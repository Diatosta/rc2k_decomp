//! `marks::draw_subcell_sprite` (0x4CBF47): a ring entry's square sprite from a subcell:
//! the place (st(0) x, st(1) y, st(2) z) turned by `g_viewMatrix` into `flt_8EB8D5` ..
//! (the depth also whole in `dword_9356C0`); unless the depth is at or below
//! `dword_8EB961` (a signed compare of the bits): the four vertices' specular
//! `scene::fog_at` of the depth, colour the entry's +0x90 | 0xFF000000, texture places from
//! subcell eax of `g_marksSubcellPlaces`, depth times `flt_8EB96D`; half the side (edx's
//! bits as a float, times `flt_624088` / depth) in `dword_8EB8B5` and `dword_8EB8B9`; the
//! corners about the projected place; `marks::clip_quad`, and unless all is outside
//! `scene::draw_fan` and `g_polygonVertexCount` 4.
//!
//! In:  st(0) .. st(2) the place (popped), eax the subcell, edx the size (a float), esi the
//!      entry
//! Out: ecx the depth's bits when too near; else as `clip_quad` / `draw_fan` leave them
//!      (eax the subcell times 16, ecx / edx its third and fourth places, edi its second)
//!
//! Callers: the ring's kinds 0x10, 0x11 (`sub_4CB64D`) and code at 0x4C55EF .. 0x4C5AB4.
//! The x87 blocks are the original's instructions over the game's variables; the fog and
//! the clip their Rust versions (the copy's calls go to their hooks); the draw through
//! `part` (1 `draw_fan`), in the game its Rust version. Refuses, before anything, a
//! subcell past the table's 0x410, a null entry, and the fog's and the clip's refusals.
//! Thunk by `three_in_hook!`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose draw goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::clip_quad::clip_quad;
use crate::scene::fog_at::fog_at;

const DEPTH_RVA: usize = rva(0x8EB8DD); // flt_8EB8DD
const WHOLE_RVA: usize = rva(0x9356C0); // dword_9356C0
const NEAR_RVA: usize = rva(0x8EB961); // dword_8EB961
const HALF_RVA: usize = rva(0x8EB8B5); // dword_8EB8B5
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const SUBCELLS_RVA: usize = rva(0x935B66); // g_marksSubcellPlaces: 0x410 of 16 bytes
const SUBCELLS: u32 = 0x410;

/// The original's 0x4CBF47 .. 0x4CBFCB: the place turned by the view.
unsafe fn turn(b: u32, p: &[[u8; 10]; 3]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{p} + 20]",
            "fld tbyte ptr [{p} + 10]",
            "fld tbyte ptr [{p}]",
            "fld dword ptr [{b} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{b} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{b} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{b} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{b} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{b} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{b} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{b} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{b} + 0x4EB8D5]",
            "fstp dword ptr [{b} + 0x4EB8D9]",
            "fist dword ptr [{b} + 0x5356C0]",
            "fstp dword ptr [{b} + 0x4EB8DD]",
            b = in(reg) b,
            p = in(reg) p.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CC009 .. 0x4CC140 (its integer moves done before): the square about
/// the projected place.
unsafe fn square(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x4EB8DD]",
            "fld dword ptr [{b} + 0x4EB8DD]",
            "fmul dword ptr [{b} + 0x4EB96D]",
            "fst dword ptr [{b} + 0x5605A8]",
            "fst dword ptr [{b} + 0x5605C8]",
            "fst dword ptr [{b} + 0x5605E8]",
            "fstp dword ptr [{b} + 0x560608]",
            "fld dword ptr [{b} + 0x4EB8B5]",
            ".byte 0xD8, 0xC9",
            "fst dword ptr [{b} + 0x4EB8B5]",
            "fstp dword ptr [{b} + 0x4EB8B9]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{b} + 0x4EB8D5]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x4EB8D9]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{b} + 0x4EB8B9]",
            "fstp dword ptr [{b} + 0x5605A4]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{b} + 0x4EB8B9]",
            "fstp dword ptr [{b} + 0x5605C4]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{b} + 0x4EB8B9]",
            "fstp dword ptr [{b} + 0x5605E4]",
            "fadd dword ptr [{b} + 0x4EB8B9]",
            "fstp dword ptr [{b} + 0x560604]",
            "fadd dword ptr [{b} + 0x22409C]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{b} + 0x4EB8B5]",
            "fstp dword ptr [{b} + 0x5605A0]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{b} + 0x4EB8B5]",
            "fstp dword ptr [{b} + 0x5605C0]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{b} + 0x4EB8B5]",
            "fstp dword ptr [{b} + 0x5605E0]",
            "fsub dword ptr [{b} + 0x4EB8B5]",
            "fstp dword ptr [{b} + 0x560600]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_subcell_sprite(entry: &Regs, place: [[u8; 10]; 3]) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.eax >= SUBCELLS {
        return Err(BadInput("a mark subcell past g_marksSubcellPlaces").into());
    }
    if g.esi == 0 {
        return Err(BadInput("a null ring entry").into());
    }
    unsafe {
        fog_at(0)?;
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let b = game_addr(0) as u32;
        turn(b, &place);
        g.ecx = game_read(DEPTH_RVA);
        if (g.ecx as i32) <= game_read::<i32>(NEAR_RVA) {
            return Ok(g);
        }
        let fog = fog_at(game_read(WHOLE_RVA))?;
        g.ecx = ((g.esi + 0x90) as *const u32).read_unaligned() | 0xFF00_0000;
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x14, fog);
            game_write(VERTICES_RVA + 0x20 * n + 0x10, g.ecx);
        }
        game_write(HALF_RVA, g.edx);
        g.eax <<= 4;
        let cell = |n: usize| game_read::<u32>(SUBCELLS_RVA + g.eax as usize + 4 * n);
        [g.ebp, g.edi, g.ecx, g.edx] = [cell(0), cell(1), cell(2), cell(3)];
        let uv = [
            (g.ebp, g.edi),
            (g.ecx, g.edi),
            (g.ecx, g.edx),
            (g.ebp, g.edx),
        ];
        for (n, (u, v)) in uv.into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        square(b);
        g.ebp = game_addr(VERTICES_RVA) as u32;
        g = clip_quad(&g)?;
        if g.carry() {
            return Ok(g);
        }
        crate::marks::draw_subcell_sprite::game_part(1, &mut g)?;
        game_write(VERTEX_COUNT_RVA, 4u32);
    }
    Ok(g)
}

/// In the game: `draw_fan`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part) };
    Ok(())
}
