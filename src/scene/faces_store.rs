//! The object faces and the two sort lists, typed (phase 7): IDA 0x9A3730 .. 0x9E3738, in
//! the image's zeroed tail (`race::buffers_store`). Names are readings unless IDA's.

use crate::race::owner::race;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Faces {
    /// +0x0 (0x9A3730): `g_faces`: the objects' faces, 0x30000 bytes (`scene::objects`, `scene::draw_object_faces`, `scene::list_object_faces`, `race::frame_draw`).
    pub faces: [Dword; 0xC000],
    /// +0x30000 (0x9D3730): `g_depthEntries`: 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`, `race::draw_view_scene`).
    pub depth_entries: [Dword; 0x2000],
    /// +0x38000 (0x9DB730): `g_sortEntries`: 0x1000 entries of 8 bytes: +0 the key, +4 the second word (`race::frame_draw`, `race::draw_view_scene`).
    pub sort_entries: [Dword; 0x2000],
    /// +0x40000 (0x9E3730): not named (`dword_9E3734` is the second).
    pub unknown_40000: [Dword; 2],
}

/// The piece, a field of `crate::race::owner::race().shared.buffers`.
pub struct InBuffers;

impl core::ops::Deref for InBuffers {
    type Target = Faces;
    fn deref(&self) -> &Faces {
        &race().shared.buffers.faces
    }
}

pub static FACES: InBuffers = InBuffers;

/// IDA 0x9A3730, the first byte of the piece.
pub const START: usize = 0x9A3730;
/// IDA 0x9E3738, the end of the piece.
pub const END: usize = 0x9E3738;

const _: () = assert!(END - START == core::mem::size_of::<Faces>());

/// Where IDA 0x9A3730 is now.
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::moved::place(START) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_buffers() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(Faces, depth_entries), 0x9D3730 - START);
        assert_eq!(offset_of!(Faces, sort_entries), 0x9DB730 - START);
        assert_eq!(offset_of!(Faces, unknown_40000), 0x9E3730 - START);
    }
}
