//! The object faces and the two sort lists (IDA 0x9A3730 .. 0x9E3738, in the image's
//! zeroed tail `race::buffers_store`). Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Faces {
    /// +0x0 (0x9A3730): `g_faces`: the objects' faces, 0x30000 bytes (`scene::objects`, `scene::draw_object_faces`, `scene::list_object_faces`, `race::frame_draw`).
    pub faces: [Dword; 0xC000],
    /// +0x30000 (0x9D3730): the original's two sort lists, no longer used: each view has its
    /// own (`race::view_scratch`). Kept for the block's layout.
    pub lists: SortLists,
}

/// The original's two sort lists (now `race::view_scratch::SortLists`, one per view).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SortLists {
    /// +0x0 (0x9D3730): `g_depthEntries`: 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`, `race::draw_view_scene`).
    pub depth_entries: [Dword; 0x2000],
    /// +0x8000 (0x9DB730): `g_sortEntries`: 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`, `race::draw_view_scene`).
    pub sort_entries: [Dword; 0x2000],
    /// +0x10000 (0x9E3730): two dwords (`dword_9E3730`, `dword_9E3734`) after the sort entries; nothing reads or writes them (an overrun space, probably).
    pub unknown_40000: [Dword; 2],
}

/// IDA 0x9A3730, the first byte of the piece.
pub const START: usize = 0x9A3730;
/// IDA 0x9E3738, the end of the piece.
pub const END: usize = 0x9E3738;

const _: () = assert!(END - START == core::mem::size_of::<Faces>());

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_buffers() {
        let lists = offset_of!(Faces, lists);
        assert_eq!(lists + offset_of!(SortLists, depth_entries), 0x30000);
        assert_eq!(lists + offset_of!(SortLists, sort_entries), 0x38000);
        assert_eq!(lists + offset_of!(SortLists, unknown_40000), 0x40000);
    }
}
