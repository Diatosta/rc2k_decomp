//! `scene::known_kind`: a known kind of car's object kind.
//!
//! Takes which of the known kinds (held to 0 ..= 0x21, signed) and returns the entry at that
//! place of the known object kinds: 0x22 dwords, one for each known kind of car. Two callers.

use crate::race::kind_store::KindTables;

pub fn known_kind(tables: &KindTables, place: u32) -> u32 {
    // Logic: `rc2k_core::scene::kinds::known_place`.
    let place = rc2k_core::scene::kinds::known_place(place);
    tables.known_object_kinds[place].get()
}
