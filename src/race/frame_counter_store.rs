//! The `framecounter` setting, 0x5EB100 (`FrameCounter.show`): shows the frame rate when not 0
//! (`race::draw_frame_rate`; `settings::read_settings` names it by address, by design).
//! Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FrameCounter {
    /// +0x0 (0x5EB100): `dword_5EB100`: the `framecounter` setting (`race::draw_frame_rate`).
    pub show: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl FrameCounter {
    pub const fn new() -> Self {
        ZeroCells::ZERO
    }
}
