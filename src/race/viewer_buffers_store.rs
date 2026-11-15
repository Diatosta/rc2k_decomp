//! The replay viewer's buffers: the panel fonts' colours, the block read from the replay file and
//! the write cursor in it, the blocks' cameras, the one undo, the two marks, the clipboard and
//! the two file handles of `race::save_viewed_replay`.

use crate::errors::BadInput;
use crate::session::{PlainBytes, PlainDword};
#[cfg(test)]
use crate::test_support::own::race;

#[derive(rc2k_derive::Cells)]
pub struct ViewerBuffers {
    /// +0x0: 256 colours the panel fonts' palettes point at (`race::viewer_palettes`).
    pub colours: [PlainDword; 256],
    /// +0x400: the bytes of the block filled, where the next frame goes (`race::replay`,
    /// `drive::write_replay_frames`, `race::open_viewed_replay`). An offset into the block.
    pub block_used: PlainDword,
    /// +0x404: the block read, 0x2810 bytes (the frame at its head: `drive::read_replay_frame`,
    /// `drive::flush_replay`, `drive::replay_frame_marks`, `race::replay`).
    pub block: [PlainDword; 0xA04],
    /// +0x2C14: 8 bytes a block, 0x708 blocks (`drive::replay_frame_marks`, `race::edit_camera`,
    /// `race::viewer`).
    pub block_cameras: PlainBytes<0x3840>,
    /// +0x6454: blocks to undo, 0 = nothing (`race::viewer`).
    pub undo_blocks: PlainDword,
    /// +0x6458: the block places of the undo, 0x3840 bytes (`race::keep_undo`, `race::undo_edit`).
    pub undo_places: [PlainDword; 0xE10],
    /// +0x9C98: the undo's cameras, 0x3840 bytes.
    pub undo_cameras: [PlainDword; 0xE10],
    /// +0xD4D8: the first block marked.
    pub mark_in: PlainDword,
    /// +0xD4DC: the last block marked.
    pub mark_out: PlainDword,
    /// +0xD4E0: 0, or the camera (1, 2) being edited.
    pub camera_edit: PlainDword,
    /// +0xD4E4: blocks on the clipboard.
    pub clip_blocks: PlainDword,
    /// +0xD4E8: the clipboard's block places, 0x3840 bytes.
    pub clip_places: [PlainDword; 0xE10],
    /// +0x10D28: the clipboard's cameras, 0x3840 bytes.
    pub clip_cameras: [PlainDword; 0xE10],
    /// +0x14568: the race's file, opened to read (`race::save_viewed_replay`).
    pub from_file: PlainDword,
    /// +0x1456c: the slot's file, made (`race::save_viewed_replay`).
    pub to_file: PlainDword,
}

/// The buffers as held by `crate::test_support::own::race().shared`.
#[cfg(test)]
pub struct InBuffers;

#[cfg(test)]
impl core::ops::Deref for InBuffers {
    type Target = ViewerBuffers;
    fn deref(&self) -> &ViewerBuffers {
        race().shared.viewer_buffers()
    }
}

#[cfg(test)]
pub static VIEWER_BUFFERS: InBuffers = InBuffers;

/// The replay block's bytes (0x2810).
pub const BLOCK_BYTES: usize = 4 * 0xA04;

impl ViewerBuffers {
    /// All bytes zero.
    pub const fn new() -> Self {
        crate::cells::zeroed_cells()
    }

    /// The replay block's bytes.
    pub fn block_bytes(&self) -> Vec<u8> {
        self.block
            .iter()
            .flat_map(|c| c.get().to_le_bytes())
            .collect()
    }

    /// `data` put in the replay block from byte `offset`; bytes past its end are refused
    /// before anything is written.
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

#[cfg(test)]
mod tests {
    use super::*;

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
