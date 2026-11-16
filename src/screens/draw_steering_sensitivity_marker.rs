//! Draws the marker of the upper left setup bar (menu item 0x10) at the percentage in the
//! player's steering sensitivity field. Takes the player and the palette to hand on at 0 percent
//! or less. Returns the percentage, or 0x10 if nothing was drawn. The conditions and bars are in
//! `setup_markers.rs`.

use crate::screens::setup_markers::{LEFT_X, Marker};

pub const MARKER: Marker = Marker::Upper {
    item: 0x10,
    x: LEFT_X,
    in_car: rc2k_core::menu::player::SetupField::SteeringSensitivity,
};
