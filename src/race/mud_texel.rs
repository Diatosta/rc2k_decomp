//! `race::mud_texel` (0x44A549): a texel of the locked mud texture (`g_surface`) by its
//! place on a 256 x 256 square: with `dword_5CEEE0` not 0 the place is halved (x, then y;
//! an odd one skips), and halved again unless it is 1 (a reading: how many times the
//! texture is smaller); then `race::mud_pixel` (0 outside the surface).
//!
//! In:  ecx = x, edx = y (0 .. 0xFF), eax
//! Out: eax = the components, 0 outside, or as it came when the place is skipped; the
//!      rest kept; no flags read (the callers test eax's top bit)
//!
//! See `race/mud.rs`. Callers: `race::muddy_texel`, `race::splash_mud`. Self-test: the
//! original and the Rust version on test surfaces (16 and 32 bits), each halving, places
//! even, odd and outside.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_read, rva};
use crate::race::mud::mud_pixel;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

pub const HALVINGS_RVA: usize = rva(0x5CEEE0); // dword_5CEEE0

/// The texel's place on the texture, or None when it is skipped (an odd place halved).
pub fn texel_place(x: u32, y: u32) -> Option<(u32, u32)> {
    // Converted: `rc2k_core::race::mud::texel_place`.
    rc2k_core::race::mud::texel_place(x, y, unsafe { game_read(HALVINGS_RVA) })
}

/// The Rust version (see the header): eax as it leaves it.
pub unsafe fn mud_texel(eax: u32, x: u32, y: u32) -> u32 {
    match texel_place(x, y) {
        Some((x, y)) => unsafe { mud_pixel(x, y) },
        None => eax,
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { mud_texel(r.eax, r.ecx, r.edx) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The places the two texel tests use: even, odd in x or y, past 64 x 48 at each halving.
pub const TEST_PLACES: [(u32, u32); 10] = [
    (0, 0),
    (4, 8),
    (5, 8),
    (4, 9),
    (2, 6),
    (62, 46),
    (64, 4),
    (8, 96),
    (0xFC, 0xBC),
    (130, 2),
];
