//! The replay viewer's buffers, typed (phase 7): IDA 0x960FA0 .. 0x975510, in the image's
//! zeroed tail (`race::buffers_store`): the panel fonts' colours, the block read from the
//! replay file and the write cursor in it, the blocks' cameras, the one undo, the two marks,
//! the clipboard and the two file handles of `race::save_viewed_replay`.
//! Names are readings unless IDA's.

use crate::errors::BadInput;
#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Bytes, Dword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewerBuffers {
    /// +0x0 (0x960FA0): `unk_960FA0`: 256 colours the panel fonts' palettes point at (`race::viewer_palettes`).
    pub colours: [Dword; 256],
    /// +0x400 (0x9613A0): `g_replayBlockCursor`: the bytes of the block filled, where the next
    /// frame goes (`race::replay`, `drive::write_replay_frames`, `race::open_viewed_replay`).
    /// The original holds the address there; an offset since 2026-10-29 (phase 13).
    pub block_used: Dword,
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

/// The piece, a field of `crate::race::owner::race().shared.buffers`.
#[cfg(test)]
pub struct InBuffers;

#[cfg(test)]
impl core::ops::Deref for InBuffers {
    type Target = ViewerBuffers;
    fn deref(&self) -> &ViewerBuffers {
        &race().shared.buffers.viewer
    }
}

#[cfg(test)]
pub static VIEWER_BUFFERS: InBuffers = InBuffers;

/// The replay block's bytes (0x2810).
pub const BLOCK_BYTES: usize = 4 * 0xA04;

impl ViewerBuffers {
    /// The replay block's bytes.
    pub fn block_bytes(&self) -> Vec<u8> {
        self.block
            .iter()
            .flat_map(|c| c.get().to_le_bytes())
            .collect()
    }

    /// `data` put in the replay block from byte `offset`; bytes past its end are refused
    /// before anything is written (the original writes on past it).
    pub fn put_in_block(&self, offset: usize, data: &[u8], what: &str) -> Result<(), BadInput> {
        match offset.checked_add(data.len()) {
            Some(end) if end <= BLOCK_BYTES => {}
            _ => {
                crate::framework::log(&format!(
                    "replay block: {what}: {:#x} bytes at {offset:#x} of {BLOCK_BYTES:#x}",
                    data.len()
                ));
                return Err(BadInput("bytes past the end of the replay block"));
            }
        }
        for (i, &b) in data.iter().enumerate() {
            let at = offset + i;
            let cell = &self.block[at / 4];
            let shift = 8 * (at % 4);
            cell.set(cell.get() & !(0xFF << shift) | u32::from(b) << shift);
        }
        Ok(())
    }
}

/// IDA 0x960FA0, the first byte of the piece.
pub const START: usize = 0x960FA0;
/// IDA 0x975510, the end of the piece.
pub const END: usize = 0x975510;

const _: () = assert!(END - START == core::mem::size_of::<ViewerBuffers>());

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(core::ptr::from_ref(&*VIEWER_BUFFERS).addr() % 4, 0);
        assert_eq!(offset_of!(ViewerBuffers, block_used), 0x400);
        assert_eq!(offset_of!(ViewerBuffers, block), 0x404);
        assert_eq!(offset_of!(ViewerBuffers, block_cameras), 0x2c14);
        assert_eq!(offset_of!(ViewerBuffers, undo_blocks), 0x6454);
        assert_eq!(offset_of!(ViewerBuffers, undo_cameras), 0x9c98);
        assert_eq!(offset_of!(ViewerBuffers, mark_in), 0xd4d8);
        assert_eq!(offset_of!(ViewerBuffers, clip_places), 0xd4e8);
        assert_eq!(offset_of!(ViewerBuffers, clip_cameras), 0x10d28);
        assert_eq!(offset_of!(ViewerBuffers, to_file), 0x1456c);
    }

    #[test]
    fn bytes_go_in_the_block_in_place() {
        let _lock = crate::test_support::test_lock();
        let kept = VIEWER_BUFFERS.block_bytes();
        let after = VIEWER_BUFFERS.block_cameras.get(0);
        VIEWER_BUFFERS.block[1].set(0xAAAA_AAAA);
        VIEWER_BUFFERS.block[2].set(0xAAAA_AAAA);
        VIEWER_BUFFERS
            .put_in_block(5, &[1, 2, 3, 4], "test")
            .unwrap();
        assert_eq!(VIEWER_BUFFERS.block[1].get(), 0x0302_01AA);
        assert_eq!(VIEWER_BUFFERS.block[2].get(), 0xAAAA_AA04);
        assert_eq!(
            &VIEWER_BUFFERS.block_bytes()[4..12],
            &[0xAA, 1, 2, 3, 4, 0xAA, 0xAA, 0xAA]
        );
        // the last byte fits; one more is refused with nothing written
        VIEWER_BUFFERS
            .put_in_block(BLOCK_BYTES - 1, &[7], "test")
            .unwrap();
        let before = VIEWER_BUFFERS.block_bytes();
        assert!(
            VIEWER_BUFFERS
                .put_in_block(BLOCK_BYTES - 1, &[8, 9], "test")
                .is_err()
        );
        assert!(
            VIEWER_BUFFERS
                .put_in_block(usize::MAX, &[8], "test")
                .is_err()
        );
        assert_eq!(VIEWER_BUFFERS.block_bytes(), before);
        assert_eq!(VIEWER_BUFFERS.block_cameras.get(0), after);
        VIEWER_BUFFERS.put_in_block(0, &kept, "test").unwrap();
    }
}
