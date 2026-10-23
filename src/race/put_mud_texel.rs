//! `race::put_mud_texel` (0x44A5A9): a texel of the locked mud texture (`g_surface`) set
//! by its place on a 256 x 256 square, the place halved as `race::mud_texel`'s; skipped
//! when an odd place is halved, or outside the surface; else `gfx::put_surface_pixel`.
//!
//! In:  eax = the components, ecx = x, edx = y (0 .. 0xFF)
//! Out: eax = the pixel put, or as it came; the rest kept; no flags read
//!
//! See `race/mud.rs`. Callers: `race::muddy_texel`, `race::splash_mud`. Refuses what
//! `put_surface_pixel` refuses (no format, no surface), before anything. Self-test: the
//! original and the Rust version on test surfaces (16 and 32 bits), each halving, places
//! even, odd and outside; the surface compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_read, rva};
use crate::gfx::put_surface_pixel::put_surface_pixel;
use crate::race::mud_texel::texel_place;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const HEIGHT_RVA: usize = rva(0x4F49A4); // g_surface.height

/// The Rust version (see the header): eax as it leaves it.
pub unsafe fn put_mud_texel(entry: &Regs) -> Result<u32, Refused> {
    let Some((x, y)) = texel_place(entry.ecx, entry.edx) else {
        return Ok(entry.eax);
    };
    unsafe {
        // Converted: `rc2k_core::race::mud::inside` (the place by `texel_place`).
        if !rc2k_core::race::mud::inside(x, y, game_read(WIDTH_RVA), game_read(HEIGHT_RVA)) {
            return Ok(entry.eax);
        }
        let put = Regs {
            ecx: x,
            edx: y,
            ..*entry
        };
        Ok(put_surface_pixel(&put)?.eax)
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { put_mud_texel(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
