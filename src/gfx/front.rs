//! The DirectDraw front and back surface handles. The platform layer sets both to 1 ("there are
//! surfaces"); the clear functions test the front one.

use crate::memory::session::PlainDword;

/// The surface handles.
#[derive(rc2k_derive::Cells)]
pub struct Surfaces {
    /// The front surface, 0 with none.
    pub front: PlainDword,
    /// The back surface.
    pub back: PlainDword,
}

impl Surfaces {
    /// The handles.
    pub const fn new() -> Self {
        Surfaces {
            front: PlainDword::new(0),
            back: PlainDword::new(0),
        }
    }
}
