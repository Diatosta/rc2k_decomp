//! The DirectDraw front and back surface handles, out of RAL.EXE's image: IDA 0x4F32B4 .. 0x4F32BC.
//! The platform layer sets both to 1 ("there are surfaces"); the clear functions test the front
//! one.

use crate::session::PlainDword;

/// The surface handles.
#[derive(rc2k_derive::Cells)]
pub struct Surfaces {
    /// `g_frontSurface` (`dword_4F32B4`): the front surface, 0 with none.
    pub front: PlainDword,
    /// `dword_4F32B8`: the back surface.
    pub back: PlainDword,
}

impl Surfaces {
    /// The handles; the first values are the image's.
    pub const fn new() -> Self {
        Surfaces {
            front: PlainDword::new(0),
            back: PlainDword::new(0),
        }
    }
}
