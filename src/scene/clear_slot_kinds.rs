//! `scene::clear_slot_kinds`: no object slot has a kind: the 0x100 dwords of the slots' kinds are
//! 0. Two callers.

pub fn clear_slot_kinds(scene: &crate::scene::owner::Scene) {
    for kind in &scene.objects.tables.get().slot_kinds {
        kind.set(0);
    }
}
