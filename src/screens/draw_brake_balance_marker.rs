//! Draws the marker of the upper right setup bar (menu item 0x11) at the percentage in the
//! player's field. Takes the player and the palette to hand on at 0 percent or less. Returns the
//! percentage, or 0x11 if nothing was drawn. The conditions and bars are in `setup_markers.rs`.

use crate::screens::setup_markers::{Marker, RIGHT_X};

pub const MARKER: Marker = Marker::Upper {
    item: 0x11,
    x: RIGHT_X,
    in_car: rc2k_core::menu::player::SetupField::BrakeBalance,
};
