//! The replay viewer's buffers, typed (phase 7): IDA 0x960FA0 .. 0x975510, in the image's
//! zeroed tail (`race::store::buffers`): the panel fonts' colours, the block read from the
//! replay file and the write cursor in it, the blocks' cameras, the one undo, the two marks,
//! the clipboard and the two file handles of `race::save_viewed_replay`.
//! Names are readings unless IDA's.

use crate::session::{Bytes, Dword};

#[repr(C)]
pub struct ViewerBuffers {
    /// +0x0 (0x960FA0): `unk_960FA0`: 256 colours the panel fonts' palettes point at (`race::viewer_palettes`).
    pub colours: [Dword; 256],
    /// +0x400 (0x9613A0): `dword_9613A0`: where the next frame goes in the block (`race::replay`, `drive::write_replay_frames`, `race::viewer`).
    pub block_cursor: Dword,
    /// +0x404 (0x9613A4): `dword_9613A4`: the block read, 0x2810 bytes (the frame at its head: `drive::read_replay_frame`, `drive::flush_replay`, `drive::replay_frame_marks`, `race::replay`).
    pub block: [Dword; 0xA04],
    /// +0x2C14 (0x963BB4): `g_blockCameras`: 8 bytes a block, 0x708 blocks (`drive::replay_frame_marks`, `race::edit_camera`, `race::viewer`).
    pub block_cameras: Bytes<0x3840>,
    /// +0x6454 (0x9673F4): `g_undoBlocks`: blocks to undo, 0 = nothing (`race::viewer`).
    pub undo_blocks: Dword,
    /// +0x6458 (0x9673F8): `g_undoPlaces`: the block places of the undo, 0x3840 bytes (`race::keep_undo`, `race::undo_edit`).
    pub undo_places: [Dword; 0xE10],
    /// +0x9C98 (0x96AC38): `g_undoCameras`: the undo's cameras, 0x3840 bytes.
    pub undo_cameras: [Dword; 0xE10],
    /// +0xD4D8 (0x96E478): `g_markIn`: the first block marked.
    pub mark_in: Dword,
    /// +0xD4DC (0x96E47C): `g_markOut`: the last block marked.
    pub mark_out: Dword,
    /// +0xD4E0 (0x96E480): `g_cameraEdit`: 0, or the camera (1, 2) being edited.
    pub camera_edit: Dword,
    /// +0xD4E4 (0x96E484): `g_clipBlocks`: blocks on the clipboard.
    pub clip_blocks: Dword,
    /// +0xD4E8 (0x96E488): `g_clipPlaces`: the clipboard's block places, 0x3840 bytes.
    pub clip_places: [Dword; 0xE10],
    /// +0x10D28 (0x971CC8): `g_clipCameras`: the clipboard's cameras, 0x3840 bytes.
    pub clip_cameras: [Dword; 0xE10],
    /// +0x14568 (0x975508): `dword_975508`: the race's file, opened to read (`race::save_viewed_replay`).
    pub from_file: Dword,
    /// +0x1456c (0x97550C): `dword_97550C`: the slot's file, made (`race::save_viewed_replay`).
    pub to_file: Dword,
}

pub struct InBuffers;

impl core::ops::Deref for InBuffers {
    type Target = ViewerBuffers;
    fn deref(&self) -> &ViewerBuffers {
        // SAFETY: the block lives as long as the program and the piece is aligned to 4 (the
        // test checks); every field is an atomic or a cell.
        unsafe { &*(piece_addr() as *const ViewerBuffers) }
    }
}

pub static VIEWER_BUFFERS: InBuffers = InBuffers;

/// IDA 0x960FA0, the first byte of the piece.
pub const START: usize = 0x960FA0;
/// IDA 0x975510, the end of the piece.
pub const END: usize = 0x975510;

const _: () = assert!(END - START == core::mem::size_of::<ViewerBuffers>());

/// Where IDA 0x960FA0 is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
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
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(ViewerBuffers, block_cursor), 0x9613A0 - START);
        assert_eq!(offset_of!(ViewerBuffers, block), 0x9613A4 - START);
        assert_eq!(offset_of!(ViewerBuffers, block_cameras), 0x963BB4 - START);
        assert_eq!(offset_of!(ViewerBuffers, undo_blocks), 0x9673F4 - START);
        assert_eq!(offset_of!(ViewerBuffers, undo_cameras), 0x96AC38 - START);
        assert_eq!(offset_of!(ViewerBuffers, mark_in), 0x96E478 - START);
        assert_eq!(offset_of!(ViewerBuffers, clip_places), 0x96E488 - START);
        assert_eq!(offset_of!(ViewerBuffers, clip_cameras), 0x971CC8 - START);
        assert_eq!(offset_of!(ViewerBuffers, to_file), 0x97550C - START);
    }
}
