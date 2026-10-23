//! `marks::squares_setup` (0x4BFEAC): the squares' frame set up: `scene::depth_buffer_on`,
//! `scene::set_texture` (`dword_5D1E10`), render states 0x11 = 2, 0x12 = 2, 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1; four vertices of `g_polygonVertices` with rhw 0x100
//! (through `dword_62400C`), colour -1, specular 0xFF000000, texture places from
//! `dword_8BC212` .. (u a, c, c, a; v b, b, d, d); `g_polygonVertexCount` 4; `flt_89CEEA` 1
//! over `flt_6241C8`.
//!
//! In:  nothing
//! Out: eax .. edx the four places, edi `g_polygonVertices`; the rest as the device calls
//!      leave them
//!
//! Caller: `sub_4BFFEC` (0x4C000F). The device calls through `part` (1 the depth buffer, 2
//! the texture, 3 a render state), in the game their Rust versions. Shadow mode runs the
//! original alone (it tells the device). Self-test: a copy whose calls go to stand-ins,
//! against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const PLACES_RVA: usize = rva(0x8BC212); // dword_8BC212 .. dword_8BC21E
const DEPTH_OF_RVA: usize = rva(0x6241C8); // flt_6241C8
const INVERSE_RVA: usize = rva(0x89CEEA); // flt_89CEEA
const STATES: [(u32, u32); 7] = [
    (2, 0x11),
    (2, 0x12),
    (4, 0x15),
    (5, 0x13),
    (6, 0x14),
    (0, 0x1C),
    (1, 0x1B),
];

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn squares_setup(entry: &Regs) -> Result<Regs, Refused> {
    unsafe { setup_with(entry, PLACES_RVA) }
}

/// The set-up with the texture places at `places` (`marks::pong_setup` has its own).
pub unsafe fn setup_with(entry: &Regs, places: usize) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        crate::marks::squares_setup::game_part(1, &mut g)?;
        g.eax = game_read(TEXTURE_RVA);
        crate::marks::squares_setup::game_part(2, &mut g)?;
        for (value, state) in STATES {
            [g.eax, g.ebx] = [value, state];
            crate::marks::squares_setup::game_part(3, &mut g)?;
        }
        g.edi = game_addr(VERTICES_RVA) as u32;
        g.eax = 0x100;
        for n in 0..4 {
            game_write(SCRATCH_RVA, g.eax);
            game_write(
                VERTICES_RVA + 0x20 * n + 0xC,
                x87::int_to_float(g.eax as i32),
            );
        }
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
            game_write(VERTICES_RVA + 0x20 * n + 0x14, 0xFF00_0000u32);
        }
        let [a, b, c, d]: [u32; 4] = game_read(places);
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        [g.eax, g.ebx, g.ecx, g.edx] = [a, b, c, d];
        game_write(COUNT_RVA, 4u32);
        game_write(INVERSE_RVA, x87::reciprocal(game_read(DEPTH_OF_RVA)));
    }
    Ok(g)
}

/// In the game: the device calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::depth_buffer_on::depth_buffer(r, true),
            2 => crate::scene::set_texture::set_texture_in_game(r),
            _ => {
                let (value, state) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(r, value, state);
            }
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { squares_setup(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
