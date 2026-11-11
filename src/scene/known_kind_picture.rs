//! `scene::known_kind_picture` (0x41F44D): a known kind of car's first picture.
//!
//! Takes: which of the known kinds (held to 0 .. 0x21, signed)
//! Returns: the number
//!
//! `ViewerAndKinds.kind_tables.known_kind_pictures[0]` at the place: 0x22 dwords, one for each known kind of car. 34 call sites, 33
//! of them in the front end's set-up.

use crate::race::kind_store::KindTables;

// ---- Rust version ----

pub fn known_kind_picture(tables: &KindTables, place: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::known_place`.
    let place = rc2k_core::scene::kinds::known_place(place);
    tables.known_kind_pictures[place].get()
}
