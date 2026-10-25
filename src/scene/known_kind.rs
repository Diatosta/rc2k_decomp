//! `scene::known_kind` (0x41F469): a known kind of car's object kind.
//!
//! In:  eax = which of the known kinds (held to 0 .. 0x21, signed)
//! Out: eax = the number; everything else preserved
//!
//! `g_knownObjectKinds` at the place: 0x22 dwords, one for each known kind of car. Two callers.

use crate::race::kind_store::KIND_TABLES;

// ---- Rust version ----

pub unsafe fn known_kind(place: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::known_place`.
    let place = rc2k_core::scene::kinds::known_place(place);
    KIND_TABLES.known_object_kinds[place].get()
}
