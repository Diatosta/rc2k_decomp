//! `scene::clear_slot_kinds` (0x426AB7): no object slot has a kind.
//!
//! In:  nothing
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! The 0x100 dwords of `g_slotKinds` = 0. Two callers.

use crate::scene::object_tables_store::OBJECT_TABLES;

// ---- Rust version ----

pub unsafe fn clear_slot_kinds() {
    for kind in &OBJECT_TABLES.slot_kinds {
        kind.set(0);
    }
}
