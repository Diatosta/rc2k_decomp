//! A view's own course working values: the four face tables `race::object_bytes` fills and
//! `race::gather_record_faces` reads, the view value `race::draw_mirror_view` keeps and the shine.
//! One per view slot (`race::view_scratch::ViewScratch::course`), so views drawn side by side do
//! not share them.

use crate::cells::ZeroCells;
use crate::session::{PlainBytes, PlainDword};

#[derive(rc2k_derive::Cells)]
pub struct ViewCourse {
    /// The four face tables (`face_table_a` .. `face_table_d`): the depth flag of an object (a,
    /// b) and its kind byte (c, d).
    pub face_table_a: PlainBytes<256>,
    pub face_table_b: PlainBytes<256>,
    pub face_table_c: PlainBytes<256>,
    pub face_table_d: PlainBytes<256>,
    /// The view value `race::draw_mirror_view` keeps.
    pub view_value_kept: PlainDword,
    /// The shine.
    pub shine: PlainDword,
}

impl ViewCourse {
    /// A view's course values at the start: the face tables empty, the shine 0x100.
    pub const fn new() -> Self {
        Self {
            face_table_a: PlainBytes::ZERO,
            face_table_b: PlainBytes::ZERO,
            face_table_c: PlainBytes::ZERO,
            face_table_d: PlainBytes::ZERO,
            view_value_kept: PlainDword::ZERO,
            shine: PlainDword::new(0x100),
        }
    }

    /// The four face tables, in order.
    pub fn tables(&self) -> [&PlainBytes<256>; 4] {
        [
            &self.face_table_a,
            &self.face_table_b,
            &self.face_table_c,
            &self.face_table_d,
        ]
    }
}
