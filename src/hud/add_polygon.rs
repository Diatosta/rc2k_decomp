//! `hud::add_polygon` (0x4B7413): `g_polygonVertices` (`g_polygonVertexCount` of them)
//! scaled from the virtual screen to the real one (x times `g_screenWidth` over
//! `flt_75C02C`, y times `g_screenHeight` over `flt_75C030`) and added to the batch: when
//! the batch holds more than 0x64 vertices or the texture asked for (`dword_75C034`) is not
//! the batch's (`dword_75C038`), `hud::flush_batch`, the texture made the batch's and the
//! shown one (`dword_6D14FE`), `hud::set_batch_texture`; the vertices copied after the
//! batch's; the indices of a triangle (and with other than three vertices a second one
//! making a quad).
//!
//! In:  nothing (`g_polygonVertices`, `g_polygonVertexCount`)
//! Out: eax the count, ecx 0, esi past the vertices, edi the triangle's indices; ebx and
//!      the rest as the flush leaves them
//!
//! Callers: `sub_4B5A86`, `hud::add_triangle`, `sub_4B7058`, `hud::add_quad`. The flush
//! and the texture through `part` (1, 2), in the game their Rust versions. Refuses a count
//! of 0 (the original's loop would run on) or past 0x10, and a batch it would write past,
//! before anything. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose two calls go to stand-ins, against the Rust version, batch counts and textures by
//! lot, at both test control words (the scaled places within `FLOAT` at extended).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::hud::batch::*;

const WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const VIRTUAL_RVA: [usize; 2] = [rva(0x75C02C), rva(0x75C030)]; // flt_75C02C, flt_75C030

/// The Rust version (see the header). What the original leaves in eax, ecx, esi and edi
/// is read by none of its callers (register liveness, 2026-10-23).
// Converted: `rc2k_core::hud::batch::{flushes, check_polygon, polygon_indices}`,
// `rc2k_core::hud::quads::screen_scaled`
pub unsafe fn add_polygon() -> Result<(), Refused> {
    use rc2k_core::hud::batch::{self, MOST_VERTICES};
    use rc2k_core::hud::quads::screen_scaled;
    unsafe {
        let count: u32 = game_read(POLYGON_COUNT_RVA);
        let texture: u32 = game_read(TEXTURE_RVA);
        let vertices: u32 = game_read(VERTEX_COUNT_RVA);
        let flushes = if count == 0 || count > MOST_VERTICES {
            false
        } else {
            batch::flushes(vertices, texture, game_read(BATCH_TEXTURE_RVA))
        };
        batch::check_polygon(count, vertices, game_read(INDEX_COUNT_RVA), flushes)?;
        let [w, h]: [u32; 2] = [game_read(WIDTH_RVA), game_read(HEIGHT_RVA)];
        let [kx, ky] = VIRTUAL_RVA.map(|r| f32::from_bits(game_read(r)));
        let f = |r: usize| f32::from_bits(game_read(r));
        for n in 0..count as usize {
            let v = POLYGON_RVA + 0x20 * n;
            game_write(v, screen_scaled(f(v), w, kx).to_bits());
            game_write(v + 4, screen_scaled(f(v + 4), h, ky).to_bits());
        }
        if flushes {
            crate::hud::flush_batch::flush_batch();
            game_write(SHOWN_TEXTURE_RVA, texture);
            game_write(BATCH_TEXTURE_RVA, texture);
            crate::hud::set_batch_texture::set_batch_texture()?;
        }
        let vc: u32 = game_read(VERTEX_COUNT_RVA);
        std::ptr::copy(
            game_addr(POLYGON_RVA) as *const u8,
            (game_addr(VERTICES_RVA) as u32 + vc * 0x20) as *mut u8,
            (count * 0x20) as usize,
        );
        let ic: u32 = game_read(INDEX_COUNT_RVA);
        let at = game_addr(INDICES_RVA) as u32 + ic * 2;
        let indices = batch::polygon_indices(vc, count);
        for (k, &i) in indices.iter().enumerate() {
            ((at + 2 * k as u32) as *mut u16).write_unaligned(i);
        }
        game_write(INDEX_COUNT_RVA, ic + indices.len() as u32);
        game_write(VERTEX_COUNT_RVA, vc + count);
    }
    Ok(())
}
