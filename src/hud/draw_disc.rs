//! `hud::draw_disc` (0x4B6C4E): the HUD's disc of 16 triangles: the ring's 16 entries (1 ..
//! 0x10 of `flt_75C188` / `flt_75C1CC`) from `g_sines` / `g_cosines` at k * 0x100, and of
//! the texture ring (`flt_75C210` / `flt_75C254`) at k * 0x100 less `dword_75C05C` times
//! `flt_75C028` times `flt_75C024` (`fistp`, & 0xFFF); then each triangle (the centre,
//! entry k + 1, entry k + 2 wrapping past 0x10) from `hud::disc_vertex` and drawn by
//! `hud::add_triangle`.
//!
//! In:  nothing
//! Out: eax the last texture angle, ebx 0x10, ecx 0x10, edx 0x10; the rest kept
//!
//! Callers: `sub_4B6B0A`, `sub_4B6B5B`, `sub_4B6BAC`, `sub_4B6BFD`. The vertices are
//! `disc_vertex`'s Rust version; the triangles through `part` (1), in the game
//! `add_triangle`'s Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose triangle call goes to a stand-in (the vertex calls to their
//! hook), against the Rust version, angles by lot.
//!
//! Converted 2026-10-21: the ring's angles and the triangles' entries through
//! `rc2k_core::hud::disc` (whole numbers; the sines are copied); the texture's turn stays
//! the x87's (its rounding follows the control word), worked out once (the original works
//! the same value out sixteen times into `dword_75C060`).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::rva;
use crate::hud::disc_vertex::disc_vertex;
use crate::hud::store::Slot;
use crate::hud::store::{game_addr, game_read, game_write};
use crate::math::x87;
use rc2k_core::hud::disc::SEGMENTS;

const RING_RVA: [Slot; 2] = [Slot::at(0x75C188), Slot::at(0x75C1CC)]; // flt_75C188, flt_75C1CC
const TEXTURE_RING_RVA: [Slot; 2] = [Slot::at(0x75C210), Slot::at(0x75C254)];
const SINES_RVA: usize = rva(0xA23AE4); // g_sines
const COSINES_RVA: usize = rva(0xA24AE4); // g_cosines
const TURN_RVA: [Slot; 3] = [Slot::at(0x75C05C), Slot::at(0x75C028), Slot::at(0x75C024)];
const SCRATCH_RVA: Slot = Slot::at(0x75C060); // dword_75C060
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices

/// `fld a; fchs; fmul b; fmul c; fistp`.
fn negated_times_times_to_int(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    unsafe {
        core::arch::asm!("fld dword ptr [{v}]", "fchs", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]", "fistp dword ptr [{o}]",
            v = in(reg) v.as_ptr(), o = in(reg) &raw mut out, out("st(0)") _, options(nostack))
    };
    out
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::disc::{ring_angles, disc_triangle}`
pub unsafe fn draw_disc() -> Result<(), Refused> {
    unsafe {
        use rc2k_core::hud::disc::{disc_triangle, ring_angles};
        // The texture's turn follows the x87's control word: the x87's.
        let t = negated_times_times_to_int(
            game_read(TURN_RVA[0]),
            game_read(TURN_RVA[1]),
            game_read(TURN_RVA[2]),
        );
        let table = |r: usize, a: u32| x87::loaded(game_read(r + 4 * a as usize));
        for (k, [a, b]) in ring_angles(t).into_iter().enumerate() {
            let n = 4 * (k + 1);
            game_write(RING_RVA[0] + n, table(SINES_RVA, a));
            game_write(RING_RVA[1] + n, table(COSINES_RVA, a));
            game_write(TEXTURE_RING_RVA[0] + n, table(SINES_RVA, b));
            game_write(TEXTURE_RING_RVA[1] + n, table(COSINES_RVA, b));
        }
        game_write(SCRATCH_RVA, t);
        let base = game_addr(VERTICES_RVA) as u32;
        for k in 0..SEGMENTS {
            for (n, e) in disc_triangle(k).into_iter().enumerate() {
                disc_vertex(e, base + 0x20 * n as u32)?;
            }
            crate::hud::add_triangle::add_triangle()?;
        }
    }
    Ok(())
}
