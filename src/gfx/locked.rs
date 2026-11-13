//! The surface-locked switch, out of RAL.EXE's image: IDA 0x4F3870 .. 0x4F3874.

use crate::session::PlainDword;

/// The surface-locked switch.
#[derive(rc2k_derive::Cells)]
pub struct Locked {
    /// `g_surfaceLocked` (`dword_4F3870`): 1 while the screen's surface is locked for drawing.
    pub surface: PlainDword,
}

impl Locked {
    /// The switch; the first value is the image's.
    pub const fn new() -> Self {
        Locked {
            surface: PlainDword::new(0),
        }
    }
}
