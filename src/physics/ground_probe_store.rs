//! The ground probe's marks: the edge test's product (a qword) and the mark that tells the ground
//! search how it was called. Plain cells: a drawing store (`session::DrawShared`), written on the
//! game's thread between frames and read by the drawing parts (the shadow's search reads the mark)
//! through their `scene::workers::DrawReads`. Names are readings unless taken from the game's own.

use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct GroundProbe {
    /// Referenced only by dead code outside any function; the port never reaches it.
    pub unused_0: PlainDword,
    /// Referenced only by dead code outside any function; the port never reaches it.
    pub unused_4: PlainDword,
    /// The product of the edge test, low then high dword (`physics::crossed_edge_contact`,
    /// `physics::ground`, `race::car_shadow`).
    pub product: [PlainDword; 2],
    /// Unused.
    pub unused_10: [PlainDword; 2],
    /// 0x40 or 0x80 by the caller of the ground search (`physics::count_ground_probe`,
    /// `physics::move_slot`, `physics::probe_ground`, `race::car_starts`).
    pub settle_mark: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl GroundProbe {
    pub const fn new() -> Self {
        use crate::memory::cells::ZeroCells;
        use crate::memory::session::PlainDword;
        Self {
            settle_mark: PlainDword::new(128),
            ..ZeroCells::ZERO
        }
    }
}
