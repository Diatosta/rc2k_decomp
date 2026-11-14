//! `scene::find_object_of_kind`: the first object (1 ..= 0xFF) of a kind, looked up in the
//! 0x100 dwords of object kinds (object 0 is never looked at). 16 call sites (level loading,
//! the carousels and car pictures, the replay cameras).

pub fn find_object_of_kind(kind: u32, scene: &crate::scene::owner::Scene) -> u32 {
    // Logic: `rc2k_core::scene::kinds::first_of_kind`.
    rc2k_core::scene::kinds::first_of_kind(
        |n| scene.objects.tables.get().object_kind(n as u32),
        kind,
    )
}
