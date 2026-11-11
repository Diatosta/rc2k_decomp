//! `scene::known_kind` (0x41F469): a known kind of car's object kind.
//!
//! Takes: which of the known kinds (held to 0 .. 0x21, signed)
//! Returns: the number
//!
//! `g_knownObjectKinds` at the place: 0x22 dwords, one for each known kind of car. Two callers.

use crate::race::kind_store::KindTables;

// ---- Rust version ----

pub fn known_kind(tables: &KindTables, place: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::known_place`.
    let place = rc2k_core::scene::kinds::known_place(place);
    tables.known_object_kinds[place].get()
}
