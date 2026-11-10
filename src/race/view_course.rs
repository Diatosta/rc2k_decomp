//! A view's own course working values (frame plan F7): the four face
//! tables `race::object_bytes` fills and `race::gather_record_faces` reads, the view value
//! `race::draw_mirror_view` keeps and the shine. One per view
//! slot (`race::view_scratch::ViewScratch::course`). In the original these were the level
//! grid's `face_table_a` .. `face_table_d` (0x63DC5C ..) and `view_value_kept` (0x63E170)
//! and the course work's `shine` (`g_shine`, 0x5CCE78), shared by every view in turn.

use crate::race::course_store::CourseWork;
use crate::race::level_grid_store::LevelGrid;
use crate::session::{Bytes, Dword};

pub struct ViewCourse {
    /// The four face tables (`byte_63DC5C` .. `byte_63DF5C`), each view's own: the depth flag
    /// of an object (a, b) and its kind byte (c, d).
    pub face_table_a: Bytes<256>,
    pub face_table_b: Bytes<256>,
    pub face_table_c: Bytes<256>,
    pub face_table_d: Bytes<256>,
    /// The view value `race::draw_mirror_view` keeps (`dword_63E170`), each view's own.
    pub view_value_kept: Dword,
    /// `g_shine` (0x5CCE78), each view's own.
    pub shine: Dword,
}

impl ViewCourse {
    /// With the image's first values.
    pub const fn new() -> Self {
        let LevelGrid {
            face_table_a,
            face_table_b,
            face_table_c,
            face_table_d,
            view_value_kept,
            ..
        } = LevelGrid::new();
        let CourseWork { shine, .. } = CourseWork::new();
        Self {
            face_table_a,
            face_table_b,
            face_table_c,
            face_table_d,
            view_value_kept,
            shine,
        }
    }

    /// The four face tables, in order.
    pub fn tables(&self) -> [&Bytes<256>; 4] {
        [
            &self.face_table_a,
            &self.face_table_b,
            &self.face_table_c,
            &self.face_table_d,
        ]
    }
}
