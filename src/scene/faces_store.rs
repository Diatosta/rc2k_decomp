//! The object faces (in the image's zeroed tail `race::buffers_store`).

use crate::memory::session::PlainDword;

#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct Faces {
    /// The objects' faces, 0x30000 bytes (`scene::objects`, `scene::draw_object_faces`,
    /// `scene::list_object_faces`, `race::frame_draw`).
    pub faces: [PlainDword; 0xC000],
}
