//! The flares' tables: five of the six flare tables (the first, the places' x, is
//! `flare_place_x`): the places' y and z, the other light, the size and the object
//! number, each 0x400 bytes. The tables hold 0x100 flares but `race::frame_draw::add_flare` runs
//! on to 0x400 (a slip kept as is: it writes into the tables that follow, so its code reaches them
//! by address).
//!
//! The race's own copy is unused (plain cells, the game's thread's): each view draws its flares in
//! its own (`race::lamp_work::LampWork::flares`, `ViewFlares`).

use crate::session::{CellKind, PlainCells};

#[derive(rc2k_derive::Cells)]
pub struct FlareTables<K: CellKind = PlainCells> {
    /// The flares' y (`race::frame_draw::add_flare`, `draw_flare`).
    pub place_y: [K::Dword; 256],
    /// The flares' z.
    pub place_z: [K::Dword; 256],
    /// The flares' colour (`other_light`).
    pub other_light: [K::Dword; 256],
    /// The flares' size.
    pub size: [K::Dword; 256],
    /// The flares' object number.
    pub object_number: [K::Dword; 256],
}

/// A view's own flares (`race::lamp_work::LampWork::flares`), which only its drawing part
/// reaches: on plain cells.
pub type ViewFlares = FlareTables<PlainCells>;

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
impl<K: CellKind> FlareTables<K>
where
    Self: crate::cells::Cells,
{
    pub const fn new() -> Self {
        crate::cells::zeroed_cells()
    }
}
