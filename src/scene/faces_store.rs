//! The object faces and the two sort lists (in the image's zeroed tail `race::buffers_store`).

use crate::memory::session::PlainDword;

#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct Faces {
    /// The objects' faces, 0x30000 bytes (`scene::objects`, `scene::draw_object_faces`,
    /// `scene::list_object_faces`, `race::frame_draw`).
    pub faces: [PlainDword; 0xC000],
    /// The two sort lists, no longer used: each view has its own (`race::view_scratch`). Kept
    /// for the block's layout.
    pub lists: SortLists,
}

/// The two sort lists (now `race::view_scratch::SortLists`, one per view).
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct SortLists {
    /// 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`,
    /// `race::draw_view_scene`).
    pub depth_entries: [PlainDword; 0x2000],
    /// 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`,
    /// `race::draw_view_scene`).
    pub sort_entries: [PlainDword; 0x2000],
    /// Two dwords after the sort entries; nothing reads or writes them.
    pub unknown_40000: [PlainDword; 2],
}
