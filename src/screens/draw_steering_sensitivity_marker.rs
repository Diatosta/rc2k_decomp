//! `screens::draw_steering_sensitivity_marker` (0x4A7027): draws the marker of the upper left setup bar (menu item 0x10), at the percentage in
//! the player's field +0x11C.
//!
//! In:  edi = player; ebx, passed on as the palette at 0 percent or less
//! Out: eax = the percentage, or 0x10 if nothing was drawn; everything else preserved
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::screens::setup_markers::{LEFT_X, Marker};

// ---- Rust version: `setup_markers::draw_marker` with this ----

pub const MARKER: Marker = Marker::Upper {
    item: 0x10,
    x: LEFT_X,
    car_offset: 0x11C,
};
