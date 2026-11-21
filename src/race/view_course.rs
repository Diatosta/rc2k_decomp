//! A view's own course working values: the four face tables `race::object_bytes` fills and
//! `race::gather_record_faces` reads, the view value `race::draw_mirror_view` keeps and the shine.
//! One per view slot (`race::view_scratch::ViewScratch::course`), so views drawn side by side do
//! not share them.

use crate::race::plain_slot::{Merge, PlainSlot, merge_run, merge_value};

/// The values of a view's course work, plain.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CourseValues {
    /// The four face tables, in order: the depth flag of an object (the first two: lit faces,
    /// other faces) and its kind byte (the last two, likewise).
    pub face_tables: [[u8; 256]; 4],
    /// The view value `race::draw_mirror_view` keeps.
    pub view_value_kept: u32,
    /// The shine.
    pub shine: u32,
}

impl CourseValues {
    /// A view's course values at the start: the face tables empty, the shine 0x100.
    pub const fn new() -> Self {
        Self {
            face_tables: [[0; 256]; 4],
            view_value_kept: 0,
            shine: 0x100,
        }
    }
}

impl Merge for CourseValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        for ((mine, now), start) in self
            .face_tables
            .iter_mut()
            .zip(&now.face_tables)
            .zip(&start.face_tables)
        {
            merge_run(mine, now, start);
        }
        merge_value(
            &mut self.view_value_kept,
            &now.view_value_kept,
            &start.view_value_kept,
        );
        merge_value(&mut self.shine, &now.shine, &start.shine);
    }
}

/// A view's course values.
pub type ViewCourse = PlainSlot<CourseValues>;
