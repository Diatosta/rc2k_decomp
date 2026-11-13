//! The ground probe's marks (IDA 0x620458 .. 0x620474; phase 7, typed): the edge test's
//! product (a qword) and the mark that tells the ground search how it was called.
//! Plain cells: a drawing store (`session::DrawShared`), written on the game's thread between
//! frames and read by the drawing parts (the shadow's search reads the mark) through their
//! `workers::DrawReads`. Names are readings unless IDA's.

use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct GroundProbe {
    /// +0x0 (0x620458): `dword_620458`: referenced only by dead code outside any function (0x450330 ..); the port never reaches it.
    pub unused_0: PlainDword,
    /// +0x4 (0x62045C): `dword_62045C`: referenced only by dead code outside any function (0x450330 ..); the port never reaches it.
    pub unused_4: PlainDword,
    /// +0x8 (0x620460): `qword_620460`: the product of the edge test, low then high dword (`physics::crossed_edge_contact`, `physics::ground`, `race::car_shadow`).
    pub product: [PlainDword; 2],
    /// +0x10 (0x620468): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_10: [PlainDword; 2],
    /// +0x18 (0x620470): `dword_620470`: 0x40 or 0x80 by the caller of the ground search (`physics::count_ground_probe`, `physics::move_slot`, `physics::probe_ground`, `race::car_starts`).
    pub settle_mark: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl GroundProbe {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        use crate::session::PlainDword;
        Self {
            settle_mark: PlainDword::new(128),
            ..ZeroCells::ZERO
        }
    }
}
