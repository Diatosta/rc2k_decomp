//! Draws the second marker of the lower right setup bar at the percentage in a driver
//! figure; always drawn. Returns the percentage. The conditions and bars are in
//! `setup_markers.rs`.

use crate::screens::setup_markers::{Marker, RIGHT_X, lower_right_installed};

pub const MARKER: Marker = Marker::Always {
    x: RIGHT_X,
    percent: lower_right_installed,
};
