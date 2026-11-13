//! Draws the marker of the lower right setup bar (menu item 0x13) at the percentage in a
//! driver figure. Takes the palette to hand on at 0 percent or less. Returns the percentage, or
//! 0x13 if nothing was drawn. The conditions and bars are in `setup_markers.rs`.

use crate::screens::setup_markers::{Marker, RIGHT_X, lower_right_chosen, lower_right_installed};

pub const MARKER: Marker = Marker::Lower {
    item: 0x13,
    x: RIGHT_X,
    chosen: lower_right_chosen,
    installed: lower_right_installed,
};
