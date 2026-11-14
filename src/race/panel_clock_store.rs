//! The panel's clock mark: the scene clock at which the panel's motion began to show
//! (`race::frame_draw::panel_motion_plain` reads it; nothing writes it). The fields keep a fixed
//! layout.

use crate::cells::ZeroCells;

use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct PanelClock {
    /// The scene clock the panel's motion shows from (`panel_motion_plain`).
    pub shown_since: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with its first values.
impl PanelClock {
    pub const fn new() -> Self {
        Self::ZERO
    }
}
