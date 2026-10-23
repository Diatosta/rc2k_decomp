//! `race::tint_to_mud_colour` (0x44A441): a colour brought toward the mud's
//! (`dword_60AD00` red, `dword_60AD04` green, `dword_60AD08` blue, whole dwords): as
//! `gfx::fade_colour`, each low byte s of ebx becoming s + ((t - s) x ebp >> 8), held to
//! 0 .. 0xFF, with t the dword (blue for the lowest byte); the top byte kept.
//!
//! In:  ebx = the colour, ebp = how far (0x100 all the way)
//! Out: eax = ebx = the colour; esi the third byte before; edx the high dword of the last
//!      `imul`; the rest kept; no flags read
//!
//! See `race/mud.rs`. One caller, `race::muddy_texel`. Self-test: the original and the
//! Rust version on colours, amounts and mud colours by lot and at the edges; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, rva};

pub const MUD_COLOUR_RVA: usize = rva(0x60AD00); // dword_60AD00 .. dword_60AD08: red, green, blue

/// The mud's colour, as its three dwords (red, green, blue).
pub fn mud_colour() -> [u32; 3] {
    unsafe { game_read(MUD_COLOUR_RVA) }
}

/// The Rust version (see the header): the registers it leaves.
pub fn tint_to_mud_colour(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::race::mud::tint`.
    let t = rc2k_core::race::mud::tint(entry.ebx, mud_colour(), entry.ebp);
    Regs {
        ebx: t.colour,
        eax: t.colour,
        esi: entry.ebx >> 16 & 0xFF,
        edx: t.high,
        ..*entry
    }
}
