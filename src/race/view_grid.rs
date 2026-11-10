//! A view's course grid (frame plan F6): the 32 x 32 course records around the view, with the
//! corner and size it was made for, one in each view's slot (`race::view_scratch`).
//!
//! The original kept it in the level grid (`race::level_grid_store`: `g_courseGrid`,
//! `dword_63E15C` .. `dword_63E178`), made again by every view whose corner moved and read
//! after the views by `race::find_sound_spots` as the last view left it. Its contents
//! depend only on the corner, the span, the course's records and the view's own marks
//! (`race::frame_draw::mark_far_cells` clears them first), so a grid per view holds what the
//! shared one held while that view drew. The views' last one (`last_drawn`) is the one the
//! sound spots read.

use crate::errors::BadInput;
use crate::race::level_grid_store::LevelGrid;
use crate::race::owner_course::Course;
use crate::session::Dword;
use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewGrid {
    /// The course records by cell (`g_courseGrid`), 0x0FFFFFFF where none; bit 31 a far
    /// cell's mark.
    pub cells: [Dword; 1024],
    /// The grid's half size (`dword_63E15C`).
    pub half: Dword,
    /// Its span (`dword_63E160`).
    pub span: Dword,
    /// The corner cell it was made for (`dword_63E174`, `dword_63E178`): made again when the
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

    /// Cell `n` as the original reads it: past the grid (up to 33 dwords) on into the level
    /// grid's variables after `g_courseGrid` (a slip kept: `race::find_sound_spots`).
    pub fn read_on(&self, course: &Course, n: u32) -> Result<u32, BadInput> {
        match self.cells.get(n as usize) {
            Some(cell) => Ok(cell.get()),
            None => crate::session::read_on_in(
                &*course.level_grid,
                &course.level_grid.course_grid,
                n as usize,
                "the variables after the course grid",
            ),
        }
    }
}

/// The slot of the view that last made its course grid (`race::frame_draw::course_grid`):
/// what the shared grid held after the views.
static LAST_DRAWN: AtomicUsize = AtomicUsize::new(0);

/// What a drawing thread's part noted (`note_drawn`), by its copy number: put into
/// `LAST_DRAWN` when its part is merged, after the parts before it (`publish_part`), so that
/// the sound spots of a view drawn beside its mirror read what they read with the mirror drawn
/// after it.
static NOTED_IN_PART: [AtomicUsize; crate::workers::COUNT] =
    [const { AtomicUsize::new(NONE) }; crate::workers::COUNT];

/// No slot noted.
const NONE: usize = usize::MAX;

/// The view in slot `slot` has made its grid.
pub fn note_drawn(slot: usize) {
    match crate::workers::index() {
        0 => LAST_DRAWN.store(slot, Ordering::Relaxed),
        n => NOTED_IN_PART[n].store(slot, Ordering::Relaxed),
    }
}

/// What part `n` (drawn on a drawing thread) noted, as the last made grid now.
pub fn publish_part(n: usize) {
    if let Some(noted) = NOTED_IN_PART.get(n) {
        let slot = noted.swap(NONE, Ordering::Relaxed);
        if slot != NONE {
            LAST_DRAWN.store(slot, Ordering::Relaxed);
        }
    }
}

/// The grid the last view made (the sound spots').
pub fn last_drawn() -> &'static ViewGrid {
    &crate::race::view_scratch::slot(LAST_DRAWN.load(Ordering::Relaxed)).grid
}

/// Every slot's corner made unknown, so each view makes its grid again (a stage's start).
pub fn reset_corners() {
    for n in 0..crate::race::view_scratch::SLOT_COUNT {
        let grid = &crate::race::view_scratch::slot(n).grid;
        grid.x.set(0x8000_0000);
        grid.z.set(0x8000_0000);
    }
}
