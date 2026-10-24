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
use crate::hud::batch::{POLYGON_COUNT_RVA, POLYGON_RVA};
use crate::hud::store::Slot;
use crate::hud::store::{game_read, game_write};
use rc2k_core::hud::quads;

pub const QUAD_RVA: Slot = Slot::at(0x75BFB0); // dword_75BFB0 .. dword_75BFE4
const SCALE_RVA: [Slot; 2] = [Slot::at(0x75C00C), Slot::at(0x75C010)]; // flt_75C00C, flt_75C010

/// The Rust version (see the header). What the original leaves in the registers is read
/// by none of its callers (register liveness, 2026-10-23).
// Converted: `rc2k_core::hud::quads::quad_vertices`
pub unsafe fn add_quad() -> Result<(), Refused> {
    unsafe {
        let q: [u32; quads::RECORD] = game_read(QUAD_RVA);
        let [a, b] = SCALE_RVA.map(|r| f32::from_bits(game_read(r)));
        let (vertices, _, _) = quads::quad_vertices(&q, a, b);
        for (n, v) in vertices.into_iter().enumerate() {
            let at = POLYGON_RVA + 0x20 * n;
            game_write(at, [v.x.to_bits(), v.y.to_bits()]);
            game_write(at + 0x10, v.colour);
            game_write(at + 0x18, [v.u.to_bits(), v.v.to_bits()]);
        }
        game_write(POLYGON_COUNT_RVA, 4u32);
        crate::hud::add_polygon::add_polygon()
    }
}
