//! `race::tint_to_mud_colour`: brings a colour toward the mud's (`StagePrefs.mud_colour`: red,
//! green and blue, whole dwords), as `gfx::fade_colour` does: each low byte of the colour moves
//! toward the matching mud component (blue for the lowest byte), held to 0 .. 0xFF; the top byte
//! is kept.
//!
//! See `race/mud.rs`.
use crate::race::owner_course::Course;

/// The mud's colour, as its three dwords (red, green, blue).
pub(super) fn mud_colour(course: &Course) -> [u32; 3] {
    crate::memory::session::read_bits(&course.stage_prefs.get().mud_colour)
}

/// `colour` brought toward the mud's by `amount`.
pub(super) fn tint_to_mud_colour(course: &Course, colour: u32, amount: u32) -> u32 {
    rc2k_core::race::mud::tint(colour, mud_colour(course), amount).colour
}
