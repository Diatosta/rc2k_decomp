//! `screens::draw_brake_balance_marker` (0x4A707F): draws the marker of the upper right setup bar (menu item 0x11), at the percentage
//! in the player's field +0x110.
//!
//! In:  edi = player; ebx, passed on as the palette at 0 percent or less
//! Out: eax = the percentage, or 0x11 if nothing was drawn; everything else preserved
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::screens::setup_markers::{Marker, RIGHT_X, marker_entry};

// ---- Rust version: `setup_markers::draw_marker` with this ----

pub const MARKER: Marker = Marker::Upper {
    item: 0x11,
    x: RIGHT_X,
    car_offset: 0x110,
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    marker_entry("screens::draw_brake_balance_marker", r, MARKER)
}
