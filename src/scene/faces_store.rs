//! The object faces and the two sort lists (IDA 0x9A3730 .. 0x9E3738, in the image's
//! zeroed tail `race::buffers_store`). Names are readings unless IDA's.

use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Faces {
    /// +0x0 (0x9A3730): `g_faces`: the objects' faces, 0x30000 bytes (`scene::objects`, `scene::draw_object_faces`, `scene::list_object_faces`, `race::frame_draw`).
    pub faces: [PlainDword; 0xC000],
    /// +0x30000 (0x9D3730): the original's two sort lists, no longer used: each view has its
    /// own (`race::view_scratch`). Kept for the block's layout.
    pub lists: SortLists,
}

/// The original's two sort lists (now `race::view_scratch::SortLists`, one per view).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SortLists {
    /// +0x0 (0x9D3730): `g_depthEntries`: 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`, `race::draw_view_scene`).
    pub depth_entries: [PlainDword; 0x2000],
    /// +0x8000 (0x9DB730): `g_sortEntries`: 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`, `race::draw_view_scene`).
    pub sort_entries: [PlainDword; 0x2000],
    /// +0x10000 (0x9E3730): two dwords (`dword_9E3730`, `dword_9E3734`) after the sort entries; nothing reads or writes them (an overrun space, probably).
    pub unknown_40000: [PlainDword; 2],
}
