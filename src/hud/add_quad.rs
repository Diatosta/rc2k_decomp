//! `hud::add_quad` (0x4B72D1): the HUD's quad record (`dword_75BFB0`: alpha, left, top,
//! right, bottom, colour, then the four corners' texture places as whole numbers) made the
//! four vertices of `g_polygonVertices` (the places over `flt_75C00C` and over
//! `flt_75C010`; the colour's low 24 bits under the alpha) and `hud::add_polygon`.
//!
//! In:  nothing
//! Out: eax the colour going in, then as `add_polygon` leaves them
//!
//! Callers: `sub_4B716A`, `sub_4B7216`, `sub_4B70EE` (reached through code at 0x4B7163,
//! 0x4B720F, 0x4B7266). The polygon through `part` (1), in the game its Rust version.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version, records by lot, at both test control words.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::hud::batch::{POLYGON_COUNT_RVA, POLYGON_RVA};
use rc2k_core::hud::quads;

pub const QUAD_RVA: usize = rva(0x75BFB0); // dword_75BFB0 .. dword_75BFE4
const SCALE_RVA: [usize; 2] = [rva(0x75C00C), rva(0x75C010)]; // flt_75C00C, flt_75C010

/// A call by place: 1 `add_polygon`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::quad_vertices`
pub unsafe fn add_quad(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let q: [u32; quads::RECORD] = game_read(QUAD_RVA);
        let [a, b] = SCALE_RVA.map(|r| f32::from_bits(game_read(r)));
        let (vertices, colour, low) = quads::quad_vertices(&q, a, b);
        for (n, v) in vertices.into_iter().enumerate() {
            let at = POLYGON_RVA + 0x20 * n;
            game_write(at, [v.x.to_bits(), v.y.to_bits()]);
            game_write(at + 0x10, v.colour);
            game_write(at + 0x18, [v.u.to_bits(), v.v.to_bits()]);
        }
        [g.eax, g.ebx] = [colour, low];
        game_write(POLYGON_COUNT_RVA, 4u32);
        part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `add_polygon`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::add_polygon as ap;
    *r = unsafe { ap::add_polygon(r, &mut ap::game_part) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { add_quad(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
