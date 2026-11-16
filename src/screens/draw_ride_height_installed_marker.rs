//! Draws the second marker of the lower left setup bar at the percentage in a driver figure;
//! always drawn. Returns the percentage. The conditions and bars are in `setup_markers.rs`.

use crate::screens::setup_markers::{LEFT_X, Marker, lower_left_installed};

pub const MARKER: Marker = Marker::Always {
    x: LEFT_X,
    percent: lower_left_installed,
};
