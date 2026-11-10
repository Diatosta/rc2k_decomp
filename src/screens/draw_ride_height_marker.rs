//! `screens::draw_ride_height_marker` (0x4A6F39): draws the marker of the lower left setup bar (menu item 0x12), at the percentage in
//! `dword_75ACA8`.
//!
//! Takes the palette to hand on at 0 percent or less. Returns the percentage, or 0x12 if
//! nothing was drawn.
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::screens::setup_markers::{LEFT_X, LOWER_LEFT_CHOSEN, LOWER_LEFT_INSTALLED, Marker};

pub const MARKER: Marker = Marker::Lower {
    item: 0x12,
    x: LEFT_X,
    chosen: LOWER_LEFT_CHOSEN,
    installed: LOWER_LEFT_INSTALLED,
};
