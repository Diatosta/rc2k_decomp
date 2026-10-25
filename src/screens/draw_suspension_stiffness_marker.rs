//! `screens::draw_suspension_stiffness_marker` (0x4A6FB0): draws the marker of the lower right setup bar (menu item 0x13), at the percentage
//! in `dword_75ACB4`.
//!
//! In:  ebx, passed on as the palette at 0 percent or less
//! Out: eax = the percentage, or 0x13 if nothing was drawn; esi = the address of the item's
//! repair flag if that was looked at; everything else preserved
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::screens::setup_markers::{LOWER_RIGHT_CHOSEN, LOWER_RIGHT_INSTALLED, Marker, RIGHT_X};

// ---- Rust version: `setup_markers::draw_marker` with this ----

pub const MARKER: Marker = Marker::Lower {
    item: 0x13,
    x: RIGHT_X,
    chosen: LOWER_RIGHT_CHOSEN,
    installed: LOWER_RIGHT_INSTALLED,
};
