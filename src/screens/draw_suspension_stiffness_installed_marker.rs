//! `screens::draw_suspension_stiffness_installed_marker` (0x4A6EF8): draws the second marker (glyph 0x77) of the lower right setup bar, at the
//! percentage in a driver figure; always drawn.
//!
//! Returns the percentage.
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::screens::setup_markers::{Marker, RIGHT_X, lower_right_installed};

pub const MARKER: Marker = Marker::Always {
    x: RIGHT_X,
    percent: lower_right_installed,
};
