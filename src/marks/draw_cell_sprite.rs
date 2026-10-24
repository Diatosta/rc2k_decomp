//! `marks::draw_cell_sprite` (0x4CB796): a ring entry's sprite: the place (st(0) x,
//! st(1) y, st(2) z) turned by `g_viewMatrix` into `flt_8EB8D5` .. (the depth also whole
//! in `dword_9356C0`); unless the depth is at or below `dword_8EB961` (a signed compare of
//! the bits): the four vertices' specular `scene::fog_at` of the depth, colour (ecx's top
//! byte | the entry's +0x90), texture places from cell eax of `g_marksCellPlaces`, depth
//! times `flt_8EB96D`; half the side (edx times `flt_624088` / depth, and that times the
//! cell's `g_marksCornerSlopes`) turned by `g_sines` / `g_cosines`[ebx] into
//! `dword_8EB8BD` ..; the corners about the projected place; `marks::clip_quad`, and unless
//! all is outside `scene::draw_fan` and `g_polygonVertexCount` 4.
//!
//! In:  st(0) .. st(2) the place (popped), eax the cell, ebx the angle, ecx the alpha (top
//!      byte), edx the size, esi the entry
//! Out: nothing (the original keeps every register but ebp, which no caller reads after it)
//!
//! Callers: the ring's kinds 1, 5, 6, 0xD, 0x13, 0x15, 0x16 (`sub_4CB64D`) and 0x4D0D80.
//! The x87 blocks are the original's instructions over the game's variables. Refuses,
//! before anything, a cell past 0x40, an angle of 0x1000 or more, a null entry, and the
//! fog's and the clip's refusals.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::clip_quad::{clip_quad, draw_fan};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::scene::fog_at::fog_at;

const DEPTH_RVA: Slot = Slot::at(0x8EB8DD); // flt_8EB8DD
const WHOLE_RVA: Slot = Slot::at(0x9356C0); // dword_9356C0
const NEAR_RVA: Slot = Slot::at(0x8EB961); // dword_8EB961
const HALF_RVA: Slot = Slot::at(0x8EB8BD); // dword_8EB8BD
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const CELLS_RVA: Slot = Slot::at(0x935766); // g_marksCellPlaces: 0x41 of 16 bytes
const CELLS: u32 = 0x41;
const ANGLES: u32 = 0x1000;

/// The original's 0x4CB796 .. 0x4CB81A: the place turned by the view.
unsafe fn turn(p: &[[u8; 10]; 3]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{p} + 20]",
            "fld tbyte ptr [{p} + 10]",
            "fld tbyte ptr [{p}]",
            "fld dword ptr [{rc} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{mk} + 0x186995]",
            "fstp dword ptr [{mk} + 0x186999]",
            "fist dword ptr [{mk} + 0x1D0780]",
            "fstp dword ptr [{mk} + 0x18699D]",
            rc = in(reg) crate::race::store::asm_base(),
            mk = in(reg) crate::marks::store::base(),
            p = in(reg) p.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4CB84C .. 0x4CBA37 (its integer moves done before but those it does
/// between the x87 steps): the square, turned, about the projected place. Gives what the
/// original loads into edi at +0x17B: `flt_8EB8C1` between its two stores.
unsafe fn square(k: u32, s: u32) -> u32 {
    let edi: u32;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{rc} + 0x224088]",
            "fdiv dword ptr [{mk} + 0x18699D]",
            "fld dword ptr [{mk} + 0x18699D]",
            "fmul dword ptr [{mk} + 0x186A2D]",
            "fst dword ptr [{bf} + 0x5605A8]",
            "fst dword ptr [{bf} + 0x5605C8]",
            "fst dword ptr [{bf} + 0x5605E8]",
            "fstp dword ptr [{bf} + 0x560608]",
            "fild dword ptr [{mk} + 0x18697D]",
            ".byte 0xD8, 0xC9",
            "fst dword ptr [{mk} + 0x18697D]",
            "fmul dword ptr [{mk} + {k} + 0x1BB7F4]",
            "fstp dword ptr [{mk} + 0x186981]",
            "mov {t}, dword ptr [{mk} + 0x18697D]",
            "xor {t}, 0x80000000",
            "mov dword ptr [{mk} + 0x186989], {t}",
            "mov {t}, dword ptr [{mk} + 0x186981]",
            "mov dword ptr [{mk} + 0x186985], {t}",
            "fld dword ptr [{mk} + 0x18697D]",
            "fld dword ptr [{mk} + 0x186981]",
            ".byte 0xD9, 0xC1",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{bf} + {s} + 0x624AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{bf} + {s} + 0x623AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x18697D]",
            ".byte 0xD9, 0xE0",
            "fmul dword ptr [{bf} + {s} + 0x623AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{bf} + {s} + 0x624AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x186981]",
            "fld dword ptr [{mk} + 0x186985]",
            "fld dword ptr [{mk} + 0x186989]",
            ".byte 0xD9, 0xC1",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{bf} + {s} + 0x624AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{bf} + {s} + 0x623AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x186985]",
            ".byte 0xD9, 0xE0",
            "fmul dword ptr [{bf} + {s} + 0x623AE4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{bf} + {s} + 0x624AE4]",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x186989]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x186995]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x186999]",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x186981]",
            "fstp dword ptr [{bf} + 0x5605A4]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x186989]",
            "fstp dword ptr [{bf} + 0x5605C4]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{mk} + 0x186981]",
            "fstp dword ptr [{bf} + 0x5605E4]",
            "fadd dword ptr [{mk} + 0x186989]",
            "fstp dword ptr [{bf} + 0x560604]",
            "fadd dword ptr [{rc} + 0x22409C]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x18697D]",
            "fstp dword ptr [{bf} + 0x5605A0]",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{mk} + 0x186985]",
            "fstp dword ptr [{bf} + 0x5605C0]",
            ".byte 0xD9, 0xC0",
            "fadd dword ptr [{mk} + 0x18697D]",
            "fstp dword ptr [{bf} + 0x5605E0]",
            "fadd dword ptr [{mk} + 0x186985]",
            "fstp dword ptr [{bf} + 0x560600]",
            rc = in(reg) crate::race::store::asm_base(),
            bf = in(reg) crate::race::store::buffers_asm_base(),
            mk = in(reg) crate::marks::store::base(),
            k = in(reg) k,
            s = in(reg) s,
            t = out(reg) edi,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
    edi
}

/// The Rust version (see the header): `cell` (eax), `angle` (ebx), `alpha` (ecx, its top
/// byte), `size` (edx), `entry` (esi). No caller reads a register it leaves.
pub unsafe fn draw_cell_sprite(
    cell: u32,
    angle: u32,
    alpha: u32,
    size: u32,
    entry: u32,
    place: [[u8; 10]; 3],
) -> Result<(), Refused> {
    if cell >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    if angle >= ANGLES {
        return Err(BadInput("an angle past the sine tables").into());
    }
    if entry == 0 {
        return Err(BadInput("a null ring entry").into());
    }
    unsafe {
        // The fog's and the clip's refusals, before anything.
        fog_at(0)?;
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        turn(&place);
        if game_read::<i32>(DEPTH_RVA) <= game_read::<i32>(NEAR_RVA) {
            return Ok(());
        }
        let fog = fog_at(game_read(WHOLE_RVA))?;
        let colour = (alpha & 0xFF00_0000) | ((entry + 0x90) as *const u32).read_unaligned();
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x14, fog);
            game_write(VERTICES_RVA + 0x20 * n + 0x10, colour);
        }
        game_write(HALF_RVA, size);
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
        square(at, 4 * angle);
        if clip_quad(game_addr(VERTICES_RVA) as u32)? {
            return Ok(());
        }
        draw_fan();
        game_write(VERTEX_COUNT_RVA, 4u32);
    }
    Ok(())
}
