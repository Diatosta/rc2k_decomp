//! One stage's course data.

use crate::memory::shared_table::SharedTable;
use std::sync::atomic::{AtomicU64, Ordering};

/// Each part's room; a course file's room (`rc2k_core::race::loading::FIRST_PART`) is smaller.
pub const ROOM: u32 = 0x100_0000;

/// One stage's course data. A part is taken from the stage's files and given back with the
/// stage; the tables are shared with the drawing threads (`SharedTable`).
pub struct CourseData {
    /// The faces, 0x40 bytes each.
    pub faces: SharedTable,
    /// The records, 0x20 bytes each.
    pub records: SharedTable,
    /// The scene points (12 bytes each) and the room after them for the points the drawing
    /// adds.
    pub points: SharedTable,
    /// The course info (0x400000 bytes: the texture infos `files\a\textinfo` and
    /// `files\b\textinfo`, the mark corners `files\m\markinfo` at +0x1C2000), filled by
    /// `level_init::load_texture_info`.
    pub info: SharedTable,
    /// Bumped when a face's first point or a scene point below the count may have
    /// changed, or the faces or points were taken or given back: what is worked out from them
    /// (`race::start_place`'s checkpoint places) is kept until then.
    pub(super) first_points_changed: AtomicU64,
}

impl CourseData {
    pub const fn new() -> Self {
        Self {
            faces: SharedTable::new(ROOM as usize, "no course faces"),
            records: SharedTable::new(ROOM as usize, "no course records"),
            points: SharedTable::new(ROOM as usize, "no scene points"),
            info: SharedTable::new(ROOM as usize, "no course info"),
            first_points_changed: AtomicU64::new(0),
        }
    }

    /// The faces' first points or the scene points have changed.
    pub(super) fn first_points_changed(&self) {
        self.first_points_changed.fetch_add(1, Ordering::Relaxed);
    }

    /// How many times the faces' first points or the scene points have changed.
    pub(super) fn first_points_changes(&self) -> u64 {
        self.first_points_changed.load(Ordering::Relaxed)
    }
}
