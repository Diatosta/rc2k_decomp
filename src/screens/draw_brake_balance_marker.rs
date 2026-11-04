//! `screens::draw_brake_balance_marker` (0x4A707F): draws the marker of the upper right setup bar (menu item 0x11), at the percentage
//! in the player's field +0x110.
//!
//! In:  edi = player; ebx, passed on as the palette at 0 percent or less
//! Out: eax = the percentage, or 0x11 if nothing was drawn; everything else preserved
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::screens::setup_markers::{Marker, RIGHT_X};

// ---- Rust version: `setup_markers::draw_marker` with this ----

pub const MARKER: Marker = Marker::Upper {
    item: 0x11,
    x: RIGHT_X,
    in_car: rc2k_core::menu::player::SetupField::BrakeBalance,
};
