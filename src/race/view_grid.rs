//! A view's course grid: the 32 x 32 course records around the view, with the corner and size it
//! was made for, one in each view's slot (`race::view_scratch`).
//!
//! A grid is made again by every view whose corner moved, and `race::find_sound_spots` reads the
//! last view's after the views are drawn. Its contents depend only on the corner, the span, the
//! course's records and the view's own marks (`race::frame_draw::mark_far_cells` clears them
//! first), so a grid per view holds what a shared one would hold while that view drew. The views'
//! last one drawn is the one the sound spots read.

use crate::cells::ZeroCells;
use crate::errors::BadInput;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells)]
pub struct ViewGrid {
    /// The course records by cell, 0x0FFFFFFF where none; bit 31 is a far cell's mark.
    pub cells: [PlainDword; 1024],
    /// The grid's half size.
    pub half: PlainDword,
    /// Its span.
    pub span: PlainDword,
    /// The corner cell it was made for (x and z): the grid is made again when the view's corner
    /// moves; `UNKNOWN_CORNER` before the first view and at a stage's start.
    pub x: PlainDword,
    pub z: PlainDword,
}

impl ViewGrid {
    /// The grid at the start: not made yet (its corner unknown, so the first view makes it).
    pub const fn new() -> Self {
        Self {
            cells: ZeroCells::ZERO,
            half: PlainDword::ZERO,
            span: PlainDword::ZERO,
            x: PlainDword::new(UNKNOWN_CORNER),
            z: PlainDword::new(UNKNOWN_CORNER),
        }
    }

    /// Cell `n`; one past the grid is refused.
    pub fn cell(&self, n: u32) -> Result<&PlainDword, BadInput> {
        self.cells
            .get(n as usize)
            .ok_or(BadInput("a course grid cell past the 32 x 32"))
    }
}

/// A corner no view reaches (a corner is a cell number less the half): the grid is made at the
/// next view.
pub const UNKNOWN_CORNER: u32 = 0x8000_0000;

/// Every slot's corner made unknown, so each view makes its grid again (a stage's start).
pub fn reset_corners() {
    for n in 0..crate::race::view_scratch::SLOT_COUNT {
        let grid = &crate::race::view_scratch::slot(n).grid;
        grid.x.set(UNKNOWN_CORNER);
        grid.z.set(UNKNOWN_CORNER);
    }
}
