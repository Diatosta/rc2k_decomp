//! A view's course grid: the 32 x 32 course records around the view, with the corner and size it
//! was made for, one in each view's slot (`race::view_scratch`).
//!
//! A grid is made again by every view whose corner moved, and `race::find_sound_spots` reads the
//! last view's after the views are drawn. Its contents depend only on the corner, the span, the
//! course's records and the view's own marks (`race::frame_draw::mark_far_cells` clears them
//! first), so a grid per view holds what a shared one would hold while that view drew. The views'
//! last one drawn is the one the sound spots read.

use crate::errors::BadInput;
use crate::race::plain_slot::{Merge, PlainSlot, merge_run, merge_value};

/// The grid, plain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewGrid {
    /// The course records by cell, 0x0FFFFFFF where none; bit 31 is a far cell's mark.
    pub cells: [u32; 1024],
    /// The grid's half size.
    pub half: u32,
    /// Its span.
    pub span: u32,
    /// The corner cell it was made for (x and z): the grid is made again when the view's corner
    /// moves; `UNKNOWN_CORNER` before the first view and at a stage's start.
    pub x: u32,
    pub z: u32,
}

impl ViewGrid {
    /// The grid at the start: not made yet (its corner unknown, so the first view makes it).
    pub const fn new() -> Self {
        Self {
            cells: [0; 1024],
            half: 0,
            span: 0,
            x: UNKNOWN_CORNER,
            z: UNKNOWN_CORNER,
        }
    }

    /// Cell `n`'s record; one past the grid is refused.
    pub fn cell(&self, n: u32) -> Result<u32, BadInput> {
        self.cells
            .get(n as usize)
            .copied()
            .ok_or(BadInput("a course grid cell past the 32 x 32"))
    }

    /// Cell `n`, to write; one past the grid is refused.
    pub fn cell_mut(&mut self, n: u32) -> Result<&mut u32, BadInput> {
        self.cells
            .get_mut(n as usize)
            .ok_or(BadInput("a course grid cell past the 32 x 32"))
    }
}

impl Merge for ViewGrid {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_run(&mut self.cells, &now.cells, &start.cells);
        merge_value(&mut self.half, &now.half, &start.half);
        merge_value(&mut self.span, &now.span, &start.span);
        merge_value(&mut self.x, &now.x, &start.x);
        merge_value(&mut self.z, &now.z, &start.z);
    }
}

/// A view's grid, in its slot.
pub type GridSlot = PlainSlot<ViewGrid>;

/// A corner no view reaches (a corner is a cell number less the half): the grid is made at the
/// next view.
pub(super) const UNKNOWN_CORNER: u32 = 0x8000_0000;

/// Every slot's corner made unknown, so each view makes its grid again (a stage's start).
pub(super) fn reset_corners(scratch: &crate::scene::scratch_store::DrawingScratch) {
    for n in 0..crate::race::view_scratch::SLOT_COUNT {
        let mut grid = scratch.slot(n).grid.borrow_mut();
        grid.x = UNKNOWN_CORNER;
        grid.z = UNKNOWN_CORNER;
    }
}
