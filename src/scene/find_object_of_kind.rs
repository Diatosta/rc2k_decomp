//! `scene::find_object_of_kind` (0x422298): the first object (1 .. 0xFF) of a kind.
//!
//! In:  eax = the kind
//! Out: eax = the object, 0 if none; the rest preserved
//!
//! `g_objectKinds` (0x5CD27C, 0x100 dwords; object 0 is never looked at). 16 call sites
//! (loadlevel_init, the carousels and car pictures, the replay cameras).

use crate::scene::object_tables_store::OBJECT_TABLES;

/// `scene::find_object_of_kind` (see the header).
pub unsafe fn find_object_of_kind(kind: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::first_of_kind`.
    rc2k_core::scene::kinds::first_of_kind(|n| OBJECT_TABLES.object_kind(n as u32), kind)
}
