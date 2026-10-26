//! `screens::draw_suspension_stiffness_installed_marker` (0x4A6EF8): draws the second marker (glyph 0x77) of the lower right setup bar, at the
//! percentage in `dword_75ACB8`; always drawn.
//!
//! In:  ebx, passed on as the palette at 0 percent or less
//! Out: eax = the percentage; everything else preserved
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::screens::setup_markers::{LOWER_RIGHT_INSTALLED_RVA, Marker, RIGHT_X, marker_entry};

// ---- Rust version: `setup_markers::draw_marker` with this ----

pub const MARKER: Marker = Marker::Always {
    x: RIGHT_X,
    percent_rva: LOWER_RIGHT_INSTALLED_RVA,
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    marker_entry(
        "screens::draw_suspension_stiffness_installed_marker",
        r,
        MARKER,
    )
}
