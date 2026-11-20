//! The replay viewer's buffers: the panel fonts' colours, the block read from the replay file and
//! the write cursor in it, the blocks' cameras, the one undo, the two marks, the clipboard and
//! the two file handles of `race::save_viewed_replay`.

use crate::errors::BadInput;
use crate::memory::session::{PlainBytes, PlainDword, PlainFlag};
#[cfg(test)]
use crate::test_support::own::race;
use rc2k_core::drive::replay_frames::{EXTENDED_FRAME_BYTES, EXTRA_TAPE_DWORDS, block_bytes};
use rc2k_core::race::replay_v2::{BLOCK_BYTES as V2_BLOCK_BYTES, MOST_BLOCKS, TAIL_DWORDS};

#[derive(rc2k_derive::Cells)]
pub struct ViewerBuffers {
    /// 256 colours the panel fonts' palettes point at (`race::viewer_palettes`).
    pub colours: [PlainDword; 256],
    /// The bytes of the block filled, where the next frame goes (`race::replay`,
    /// `drive::write_replay_frames`, `race::open_viewed_replay`). An offset into the block.
    pub block_used: PlainDword,
    /// The block read, 0x2810 bytes (the frame at its head: `drive::read_replay_frame`,
    /// `drive::flush_replay`, `drive::replay_frame_marks`, `race::replay`).
    pub block: [PlainDword; 0xA04],
    /// 8 bytes a block, 0x708 blocks (`drive::replay_frame_marks`, `race::edit_camera`,
    /// `race::viewer`).
    pub block_cameras: PlainBytes<{ 8 * MOST_BLOCKS as usize }>,
    /// Blocks to undo, 0 = nothing (`race::viewer`).
    pub undo_blocks: PlainDword,
    /// The block places of the undo, 0x3840 bytes (`race::keep_undo`, `race::undo_edit`).
    pub undo_places: [PlainDword; 2 * MOST_BLOCKS as usize],
    /// The undo's cameras, 0x3840 bytes.
    pub undo_cameras: [PlainDword; 2 * MOST_BLOCKS as usize],
    /// The first block marked.
    pub mark_in: PlainDword,
    /// The last block marked.
    pub mark_out: PlainDword,
    /// 0, or the camera (1, 2) being edited.
    pub camera_edit: PlainDword,
    /// Blocks on the clipboard.
    pub clip_blocks: PlainDword,
    /// The clipboard's block places, 0x3840 bytes.
    pub clip_places: [PlainDword; 2 * MOST_BLOCKS as usize],
    /// The clipboard's cameras, 0x3840 bytes.
    pub clip_cameras: [PlainDword; 2 * MOST_BLOCKS as usize],
    /// The race's file, opened to read (`race::save_viewed_replay`).
    pub from_file: PlainDword,
    /// The slot's file, made (`race::save_viewed_replay`).
    pub to_file: PlainDword,
    /// What follows the block's plain frame in a replay with the extra tape: the inputs of cars
    /// 2 and 3 (`EXTRA_TAPE_BYTES`). Past the original's record, so it is not a place the
    /// camera marks' reads past their table reach.
    pub block_extra: [PlainDword; EXTRA_TAPE_DWORDS],
    /// The replay in the block has the extra tape: its header's version
    /// (`rc2k_core::race::replay::HEADER_VERSION_EXTRA_TAPE`).
    pub extra_tape: PlainFlag,
    /// What follows the extra tape in a replay of this project's own format
    /// (`rc2k_core::race::replay_v2`): the tail, `TAIL_DWORDS` dwords.
    pub block_tail: [PlainDword; TAIL_DWORDS],
    /// The replay in the block is of this project's own format (version 2): its block is the
    /// plain frame, the extra tape and the tail.
    pub v2: PlainFlag,
}

/// The buffers as held by `crate::test_support::own::race().shared`.
#[cfg(test)]
pub(super) struct InBuffers;

#[cfg(test)]
impl core::ops::Deref for InBuffers {
    type Target = ViewerBuffers;
    fn deref(&self) -> &ViewerBuffers {
        race().shared.viewer_buffers()
    }
}

#[cfg(test)]
static VIEWER_BUFFERS: InBuffers = InBuffers;

/// The replay block's bytes (0x2810), `EXTENDED_FRAME_BYTES` with the extra tape.
pub const BLOCK_BYTES: usize = 4 * 0xA04;

impl ViewerBuffers {
    /// All bytes zero.
    pub const fn new() -> Self {
        crate::memory::cells::zeroed_cells()
    }

    /// The bytes the block holds: with the extra tape, the plain frame and the tape.
    pub fn block_len(&self) -> usize {
        if self.v2.is_set() {
            V2_BLOCK_BYTES
        } else {
            block_bytes(self.extra_tape.is_set())
        }
    }

    /// The replay block's bytes (`block_len`).
    pub fn block_bytes(&self) -> Vec<u8> {
        let tail = self.v2.is_set().then_some(&self.block_tail);
        self.block
            .iter()
            .chain(
                (self.extra_tape.is_set() || self.v2.is_set())
                    .then_some(&self.block_extra)
                    .into_iter()
                    .flatten(),
            )
            .chain(tail.into_iter().flatten())
            .flat_map(|c| c.get().to_le_bytes())
            .collect()
    }

    /// `data` put in the replay block from byte `offset`; bytes past its end (`block_len`) are
    /// refused before anything is written.
    pub fn put_in_block(&self, offset: usize, data: &[u8], what: &str) -> Result<(), BadInput> {
        let len = self.block_len();
        match offset.checked_add(data.len()) {
            Some(end) if end <= len => {}
            _ => {
                crate::program_log::log(&format!(
                    "replay block: {what}: {:#x} bytes at {offset:#x} of {len:#x}",
                    data.len()
                ));
                return Err(BadInput("bytes past the end of the replay block"));
            }
        }
        for (i, &b) in data.iter().enumerate() {
            let at = offset + i;
            let word = at / 4;
            let cell = match self.block.get(word) {
                Some(cell) => cell,
                None => match self.block_extra.get(word - self.block.len()) {
                    Some(cell) => cell,
                    None => &self.block_tail[word - self.block.len() - self.block_extra.len()],
                },
            };
            let shift = 8 * (at % 4);
            cell.set(cell.get() & !(0xFF << shift) | u32::from(b) << shift);
        }
        Ok(())
    }
}

const _: () = assert!(BLOCK_BYTES + 4 * EXTRA_TAPE_DWORDS == EXTENDED_FRAME_BYTES);

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

    #[test]
    fn the_extra_tape_follows_the_plain_frame() {
        let _lock = crate::test_support::test_lock();
        let kept = VIEWER_BUFFERS.block_bytes();
        assert_eq!(kept.len(), BLOCK_BYTES);
        VIEWER_BUFFERS.extra_tape.set(true);
        let extra_kept = VIEWER_BUFFERS.block_bytes();
        assert_eq!(extra_kept.len(), EXTENDED_FRAME_BYTES);
        VIEWER_BUFFERS
            .put_in_block(BLOCK_BYTES - 2, &[1, 2, 3, 4], "test")
            .unwrap();
        assert_eq!(VIEWER_BUFFERS.block[0xA03].get() >> 16, 0x0201);
        assert_eq!(VIEWER_BUFFERS.block_extra[0].get() & 0xFFFF, 0x0403);
        let bytes = VIEWER_BUFFERS.block_bytes();
        assert_eq!(&bytes[BLOCK_BYTES - 2..BLOCK_BYTES + 2], &[1, 2, 3, 4]);
        assert!(
            VIEWER_BUFFERS
                .put_in_block(EXTENDED_FRAME_BYTES - 1, &[1, 2], "test")
                .is_err()
        );
        VIEWER_BUFFERS.put_in_block(0, &extra_kept, "test").unwrap();
        VIEWER_BUFFERS.extra_tape.set(false);
        VIEWER_BUFFERS.put_in_block(0, &kept, "test").unwrap();
        // Without the extra tape a block of the longer size is refused.
        assert!(VIEWER_BUFFERS.put_in_block(0, &extra_kept, "test").is_err());
    }
}
