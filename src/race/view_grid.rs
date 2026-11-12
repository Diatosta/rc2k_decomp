//! A view's course grid (frame plan F6): the 32 x 32 course records around the view, with the
//! corner and size it was made for, one in each view's slot (`race::view_scratch`).
//!
//! The original kept it in the level grid (`race::level_grid_store`: `g_courseGrid`,
//! `grid_half` .. `grid_z`), made again by every view whose corner moved and read
//! after the views by `race::find_sound_spots` as the last view left it. Its contents
//! depend only on the corner, the span, the course's records and the view's own marks
//! (`race::frame_draw::mark_far_cells` clears them first), so a grid per view holds what the
//! shared one held while that view drew. The views' last one (`last_drawn`) is the one the
//! sound spots read.

use crate::errors::BadInput;
use crate::race::level_grid_store::LevelGrid;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewGrid {
    /// The course records by cell (`g_courseGrid`), 0x0FFFFFFF where none; bit 31 a far
    /// cell's mark.
    pub cells: [Dword; 1024],
    /// The grid's half size (`grid_half`).
    pub half: Dword,
    /// Its span (`grid_span`).
    pub span: Dword,
    /// The corner cell it was made for (`grid_x`, `grid_z`): made again when the
    /// view's moves; 0x80000000 at a stage's start (`race::reset_refresh`).
    pub x: Dword,
    pub z: Dword,
}

impl ViewGrid {
    /// With the image's first values.
    pub const fn new() -> Self {
        let LevelGrid {
            grid_half,
            grid_span,
            grid_x,
            grid_z,
            course_grid,
            ..
        } = LevelGrid::new();
        Self {
            cells: course_grid,
            half: grid_half,
            span: grid_span,
            x: grid_x,
            z: grid_z,
        }
    }

    /// Cell `n`; one past the grid refused.
    pub fn cell(&self, n: u32) -> Result<&Dword, BadInput> {
        self.cells
            .get(n as usize)
            .ok_or(BadInput("a course grid cell past the 32 x 32"))
    }
}

/// Every slot's corner made unknown, so each view makes its grid again (a stage's start).
pub fn reset_corners() {
    for n in 0..crate::race::view_scratch::SLOT_COUNT {
        let grid = &crate::race::view_scratch::slot(n).grid;
        grid.x.set(0x8000_0000);
        grid.z.set(0x8000_0000);
    }
}
