//! The numbers of the course faces that are lit (a contact touched them, or a lamp burns): the
//! physics' step adds them, `drive::animate_lit_faces` ages them and the drawing parts gather the
//! lamps from them.

use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct LitFaces {
    /// The numbers of the course faces lit (`physics::touch_face`, `drive::animate_lit_faces`,
    /// `race::frame_draw::lights`).
    pub faces: [PlainDword; 256],
    /// How many faces are in the list.
    pub count: PlainDword,
}

/// The list, empty.
impl LitFaces {
    pub const fn new() -> Self {
        use crate::memory::cells::ZeroCells;
        <Self as ZeroCells>::ZERO
    }
}
