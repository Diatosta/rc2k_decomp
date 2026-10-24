//! `race::tint_to_mud_colour` (0x44A441): a colour brought toward the mud's
//! (`dword_60AD00` red, `dword_60AD04` green, `dword_60AD08` blue, whole dwords): as
//! `gfx::fade_colour`, each low byte s of ebx becoming s + ((t - s) x ebp >> 8), held to
//! 0 .. 0xFF, with t the dword (blue for the lowest byte); the top byte kept.
//!
//! In:  the colour (ebx), how far (ebp; 0x100 all the way)
//! Out: the colour (eax = ebx); its one caller reads nothing else (conversion phase 3,
//!      2026-10-24: esi, edx dropped)
//!
//! See `race/mud.rs`. One caller, `race::muddy_texel`. Self-test: the original and the
//! Rust version on colours, amounts and mud colours by lot and at the edges; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, rva};

pub const MUD_COLOUR_RVA: usize = rva(0x60AD00); // dword_60AD00 .. dword_60AD08: red, green, blue

/// The mud's colour, as its three dwords (red, green, blue).
pub fn mud_colour() -> [u32; 3] {
    unsafe { game_read(MUD_COLOUR_RVA) }
}

/// The Rust version (see the header): `colour` brought toward the mud's by `amount`.
pub fn tint_to_mud_colour(colour: u32, amount: u32) -> u32 {
    // Converted: `rc2k_core::race::mud::tint`.
    rc2k_core::race::mud::tint(colour, mud_colour(), amount).colour
}
