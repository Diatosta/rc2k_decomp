//! `marks::hexagons_setup` (0x4BBE62): the hexagons' frame set up: their two axes
//! (`flt_814C1A`, `flt_814C26`) times `flt_814BC2` into `unk_814C6E` / `unk_814C7A` and those
//! times `flt_82E286` .. into `unk_814C86` / `unk_814C92`; `scene::depth_buffer_on`,
//! `scene::set_texture` (`dword_5D5A24`), render states 0x11 = 1, 0x12 = 1, 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1, 3 = 1; six vertices of `g_polygonVertices` with rhw 0x100
//! (through `dword_62400C`), colour -1, specular 0xFF000000, texture places 0 /
//! `dword_82E2A2` (u 0, a, a, 0; v 0, 0, a, a); `g_polygonVertexCount` 6; `flt_814C02` 1 over
//! `flt_6241C8`.
//!
//! In:  nothing
//! Out: eax `dword_82E2A2`, esi `unk_814C7A`, edi `g_polygonVertices`; the rest as the
//!      device calls leave them
//!
//! Caller: `sub_4BC0B1` (0x4BC0BF). The x87 run is the original's instructions over the
//! game's variables; the device calls through `part` (1 the depth buffer, 2 the texture,
//! 3 a render state), in the game their Rust versions. Shadow mode runs the original alone
//! (it tells the device). Self-test: a copy whose calls go to stand-ins, against the Rust
//! version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

const TEXTURE_RVA: usize = rva(0x5D5A24); // dword_5D5A24
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const SIDE_RVA: usize = rva(0x82E2A2); // dword_82E2A2
const DEPTH_OF_RVA: usize = rva(0x6241C8); // flt_6241C8
const INVERSE_RVA: usize = rva(0x814C02); // flt_814C02
const STATES: [(u32, u32); 8] = [
    (1, 0x11),
    (1, 0x12),
    (4, 0x15),
    (5, 0x13),
    (6, 0x14),
    (0, 0x1C),
    (1, 0x1B),
    (1, 3),
];

/// The original's 0x4BBE62 .. 0x4BBF1A: the axes scaled.
unsafe fn axes(b: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x414C1A]",
            "fmul dword ptr [{b} + 0x414BC2]",
            "fstp dword ptr [{b} + 0x414C6E]",
            "fld dword ptr [{b} + 0x414C1E]",
            "fmul dword ptr [{b} + 0x414BC2]",
            "fstp dword ptr [{b} + 0x414C72]",
            "fld dword ptr [{b} + 0x414C22]",
            "fmul dword ptr [{b} + 0x414BC2]",
            "fstp dword ptr [{b} + 0x414C76]",
            "fld dword ptr [{b} + 0x414C26]",
            "fmul dword ptr [{b} + 0x414BC2]",
            "fstp dword ptr [{b} + 0x414C7A]",
            "fld dword ptr [{b} + 0x414C2A]",
            "fmul dword ptr [{b} + 0x414BC2]",
            "fstp dword ptr [{b} + 0x414C7E]",
            "fld dword ptr [{b} + 0x414C2E]",
            "fmul dword ptr [{b} + 0x414BC2]",
            "fstp dword ptr [{b} + 0x414C82]",
            "fld dword ptr [{b} + 0x414C6E]",
            "fmul dword ptr [{b} + 0x42E286]",
            "fstp dword ptr [{b} + 0x414C86]",
            "fld dword ptr [{b} + 0x414C72]",
            "fmul dword ptr [{b} + 0x42E28A]",
            "fstp dword ptr [{b} + 0x414C8A]",
            "fld dword ptr [{b} + 0x414C76]",
            "fmul dword ptr [{b} + 0x42E28E]",
            "fstp dword ptr [{b} + 0x414C8E]",
            "fld dword ptr [{b} + 0x414C7A]",
            "fmul dword ptr [{b} + 0x42E292]",
            "fstp dword ptr [{b} + 0x414C92]",
            "fld dword ptr [{b} + 0x414C7E]",
            "fmul dword ptr [{b} + 0x42E296]",
            "fstp dword ptr [{b} + 0x414C96]",
            "fld dword ptr [{b} + 0x414C82]",
            "fmul dword ptr [{b} + 0x42E29A]",
            "fstp dword ptr [{b} + 0x414C9A]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn hexagons_setup(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        axes(game_addr(0) as u32);
        [g.esi, g.edi] = [
            game_addr(rva(0x814C7A)) as u32,
            game_addr(rva(0x814C92)) as u32,
        ];
        crate::marks::hexagons_setup::game_part(1, &mut g)?;
        g.eax = game_read(TEXTURE_RVA);
        crate::marks::hexagons_setup::game_part(2, &mut g)?;
        for (value, state) in STATES {
            [g.eax, g.ebx] = [value, state];
            crate::marks::hexagons_setup::game_part(3, &mut g)?;
        }
        g.edi = game_addr(VERTICES_RVA) as u32;
        g.eax = 0x100;
        for n in 0..6 {
            game_write(SCRATCH_RVA, g.eax);
            game_write(
                VERTICES_RVA + 0x20 * n + 0xC,
                x87::int_to_float(g.eax as i32),
            );
        }
        for n in 0..6 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
            game_write(VERTICES_RVA + 0x20 * n + 0x14, 0xFF00_0000u32);
        }
        g.eax = game_read(SIDE_RVA);
        for (n, (u, v)) in [(0, 0), (g.eax, 0), (g.eax, g.eax), (0, g.eax)]
            .into_iter()
            .enumerate()
        {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        game_write(COUNT_RVA, 6u32);
        game_write(INVERSE_RVA, x87::reciprocal(game_read(DEPTH_OF_RVA)));
    }
    Ok(g)
}

/// In the game: the device calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => crate::scene::depth_buffer_on::depth_buffer(true),
        2 => crate::scene::set_texture::set_texture(r.eax),
        _ => {
            let (value, state) = (r.eax, r.ebx);
            crate::scene::set_render_state::set_render_state(value, state);
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { hexagons_setup(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
