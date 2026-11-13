//! A view's own course working values (frame plan F7): the four face
//! tables `race::object_bytes` fills and `race::gather_record_faces` reads, the view value
//! `race::draw_mirror_view` keeps and the shine. One per view
//! slot (`race::view_scratch::ViewScratch::course`). In the original these were the level
//! grid's `face_table_a` .. `face_table_d` (0x63DC5C ..) and `view_value_kept` (0x63E170)
//! and the course work's `shine` (`g_shine`, 0x5CCE78), shared by every view in turn.

use crate::cells::ZeroCells;
use crate::session::{PlainBytes, PlainDword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewCourse {
    /// The four face tables (`face_table_a` .. `face_table_d`), each view's own: the depth flag
    /// of an object (a, b) and its kind byte (c, d).
    pub face_table_a: PlainBytes<256>,
    pub face_table_b: PlainBytes<256>,
    pub face_table_c: PlainBytes<256>,
    pub face_table_d: PlainBytes<256>,
    /// The view value `race::draw_mirror_view` keeps (`view_value_kept`), each view's own.
    pub view_value_kept: PlainDword,
    /// `g_shine` (0x5CCE78), each view's own.
    pub shine: PlainDword,
}

impl ViewCourse {
    /// A view's course values as the game starts them: the face tables empty, the shine 0x100.
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
