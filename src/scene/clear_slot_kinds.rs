//! `scene::clear_slot_kinds`: no object slot has a kind: the 0x100 dwords of the slots' kinds are
//! 0. Two callers.

pub fn clear_slot_kinds(scene: &crate::scene::owner::Scene) {
    scene.texture_levels_mut().slot_kinds.fill(0);
}
