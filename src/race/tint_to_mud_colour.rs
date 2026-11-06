//! `race::tint_to_mud_colour` (0x44A441): a colour brought toward the mud's
//! (`dword_60AD00` red, `dword_60AD04` green, `dword_60AD08` blue, whole dwords): as
//! `gfx::fade_colour`, each low byte s of ebx becoming s + ((t - s) x ebp >> 8), held to
//! 0 .. 0xFF, with t the dword (blue for the lowest byte); the top byte kept.
//!
//! In:  the colour (ebx), how far (ebp; 0x100 all the way)
//! Out: the colour (eax = ebx); its one caller reads nothing else (conversion phase 3,
//!      2026-10-24: esi, edx dropped)
//!
//! See `race/mud.rs`. One caller, `race::muddy_texel`.
use crate::race::owner::race;

/// The mud's colour, as its three dwords (red, green, blue).
pub fn mud_colour() -> [u32; 3] {
    crate::session::read_bits(&race().course.stage_prefs.mud_colour)
}

/// The Rust version (see the header): `colour` brought toward the mud's by `amount`.
pub fn tint_to_mud_colour(colour: u32, amount: u32) -> u32 {
    // Converted: `rc2k_core::race::mud::tint`.
    rc2k_core::race::mud::tint(colour, mud_colour(), amount).colour
}
