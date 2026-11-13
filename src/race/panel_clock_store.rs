//! The panel's clock mark (IDA 0x62290C .. 0x622910; phase 7, typed): the scene clock at
//! which the panel's motion began to show (`race::frame_draw::panel_motion` reads it; no
//! code of the game writes it). The fields are in the image's order. Names are readings unless IDA's.

use crate::cells::ZeroCells;

use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct PanelClock {
    /// +0x0 (0x62290C): `dword_62290C`: the scene clock the panel's motion shows from (`race::panel_motion`).
    pub shown_since: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl PanelClock {
    pub const fn new() -> Self {
        Self::ZERO
    }
}
