//! The ground probe's mark: tells the ground search how it was called. A plain cell: a drawing
//! store (`session::DrawShared`), written on the game's thread between frames and read by the
//! drawing parts (the shadow's search reads the mark) through their `scene::workers::DrawReads`.
//! Names are readings unless taken from the game's own.

use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct GroundProbe {
    /// 0x40 or 0x80 by the caller of the ground search (`physics::count_ground_probe`,
    /// `physics::move_slot`, `physics::probe_ground`, `race::car_starts`).
    pub settle_mark: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl GroundProbe {
    pub const fn new() -> Self {
        Self {
            settle_mark: PlainDword::new(128),
        }
    }
}
