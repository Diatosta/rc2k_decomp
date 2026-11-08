//! `screens::draw_suspension_stiffness_installed_marker` (0x4A6EF8): draws the second marker (glyph 0x77) of the lower right setup bar, at the
//! percentage in `dword_75ACB8`; always drawn.
//!
//! Returns the percentage.
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::screens::setup_markers::{LOWER_RIGHT_INSTALLED, Marker, RIGHT_X};

// ---- Rust version: `setup_markers::draw_marker` with this ----

pub const MARKER: Marker = Marker::Always {
    x: RIGHT_X,
    percent: LOWER_RIGHT_INSTALLED,
};
