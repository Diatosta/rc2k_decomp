//! `screens::draw_ride_height_installed_marker` (0x4A6EB7): draws the second marker (glyph 0x77) of the lower left setup bar, at the
//! percentage in `dword_75ACAC`; always drawn.
//!
//! In:  ebx, passed on as the palette at 0 percent or less
//! Out: eax = the percentage; everything else preserved
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::screens::setup_markers::{LEFT_X, LOWER_LEFT_INSTALLED, Marker};

// ---- Rust version: `setup_markers::draw_marker` with this ----

pub const MARKER: Marker = Marker::Always {
    x: LEFT_X,
    percent: LOWER_LEFT_INSTALLED,
};
