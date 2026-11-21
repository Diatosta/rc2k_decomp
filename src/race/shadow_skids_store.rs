//! The skid marks' work: the least grip that lays a mark, the lift of a point, a wheel's place,
//! the ring of links (8 bytes each), the next point, the cars' bit and the ring of points (0xC
//! bytes each). Plain cells: a drawing store (`session::DrawShared`), written on the game's thread
//! between frames and read by the drawing parts (the skid marks) through their
//! `scene::workers::DrawReads`. The tables are 40 KB of byte cells: kept as cells, since a copy per
//! drawing scope would cost more than it saves for the one reader.

use crate::memory::cells::ZeroCells;

use crate::memory::session::{PlainBytes, PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ShadowSkids {
    /// The least grip that lays a skid mark (`race::skid_laying`).
    pub skid_grip: PlainFloat,
    /// The lift of a skid point (`race::skidmarks`).
    pub skid_lift: PlainFloat,
    /// The wheel's place, three dwords (`race::skid_laying`).
    pub skid_place: [PlainDword; 3],
    /// 0x400 links of 8 bytes each (`race::skid_laying`, `race::skidmarks`,
    /// `race::stage_prefs`).
    pub skid_links: PlainBytes<0x2000>,
    /// The next point of the ring (`race::skid_laying`, `race::stage_prefs`).
    pub skid_next: PlainDword,
    /// The bit of the car being laid for.
    pub skid_cars: PlainDword,
    /// The points, 0xC bytes each (0x400 are used; the room is 0x800).
    pub skid_points: PlainBytes<0x6000>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ShadowSkids {
    pub const fn new() -> Self {
        Self {
            skid_grip: PlainFloat::new(8.0),
            skid_lift: PlainFloat::new(4096.0),
            ..ZeroCells::ZERO
        }
    }
}
