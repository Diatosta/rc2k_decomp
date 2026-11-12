//! `race::tint_to_mud_colour` (0x44A441): a colour brought toward the mud's (`mud_colour` red,
//! `StagePrefs.mud_colour[1]` green, `StagePrefs.mud_colour[2]` blue, whole dwords): as `gfx::fade_colour`, each low byte s
//! of the colour becoming s +, held to 0 .. 0xFF, with t the dword (blue for the lowest byte); the top
//! byte kept.
//!
//! See `race/mud.rs`. One caller, `race::muddy_texel`.
use crate::race::owner_course::Course;

/// The mud's colour, as its three dwords (red, green, blue).
pub fn mud_colour(course: &Course) -> [u32; 3] {
    crate::session::read_bits(&course.stage_prefs.get().mud_colour)
}

/// `colour` brought toward the mud's by `amount`.
pub fn tint_to_mud_colour(course: &Course, colour: u32, amount: u32) -> u32 {
    // Converted: `rc2k_core::race::mud::tint`.
    rc2k_core::race::mud::tint(colour, mud_colour(course), amount).colour
}
