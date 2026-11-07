//! `scene::clear_slot_kinds` (0x426AB7): no object slot has a kind.
//!
//! In:  nothing
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! The 0x100 dwords of `g_slotKinds` = 0. Two callers.

// ---- Rust version ----

pub fn clear_slot_kinds(scene: &crate::scene::owner::Scene) {
    for kind in &scene.objects.tables.slot_kinds {
        kind.set(0);
    }
}
