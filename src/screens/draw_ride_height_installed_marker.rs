//! `screens::draw_ride_height_installed_marker` (0x4A6EB7): draws the second marker (glyph 0x77) of the lower left setup bar, at the
//! percentage in a driver figure; always drawn.
//!
//! Returns the percentage.
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::screens::setup_markers::{LEFT_X, LOWER_LEFT_INSTALLED, Marker};

pub const MARKER: Marker = Marker::Always {
    x: LEFT_X,
    percent: LOWER_LEFT_INSTALLED,
};
