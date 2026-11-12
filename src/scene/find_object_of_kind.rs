//! `scene::find_object_of_kind` (0x422298): the first object (1 .. 0xFF) of a kind.
//!
//! Takes: the kind
//!
//! `g_objectKinds` (0x5CD27C, 0x100 dwords; object 0 is never looked at). 16 call sites
//! (loadlevel_init, the carousels and car pictures, the replay cameras).

/// `scene::find_object_of_kind` (see the header).
pub fn find_object_of_kind(kind: u32, scene: &crate::scene::owner::Scene) -> u32 {
    // Converted: `rc2k_core::scene::kinds::first_of_kind`.
    rc2k_core::scene::kinds::first_of_kind(
        |n| scene.objects.tables.get().object_kind(n as u32),
        kind,
    )
}
