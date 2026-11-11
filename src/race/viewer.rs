//! The viewer plays a replay from `var\tmprpl` (the last race's, or a saved one copied
//! there first), block by block (`replay_block`, 0 .. `index_block_count`; 100 frames a block,
//! the frame in `replay_frame`), and edits it: two marks (`mark_in`, `mark_out`),
//! cut, copy and paste of blocks (the clipboard `clip_places`, `clip_cameras`), one undo
//! (`undo_blocks` ..), the cameras of each block (`block_cameras`: 8 bytes a block, the
//! first 8 of the block's data), and saving the result as a replay slot.
//!
//! The tests: a copy of the original whose calls go to `install::stand_ins` (`copy`), noted by
//! `note` with the call's four register values and answered by the test's `answer`; the Rust side calls
//! `test_part`, which does the same.

use crate::errors::BadInput;
use crate::math::cells::Cells;

// ---- the replay being viewed ----

pub const LAST_RACE: u32 = 0xDE; // the slot of `var\tmprpl` as it is
pub const TABLE_LEN: usize = 0x3840; // 0x708 blocks of 8 bytes
/// A byte of the blocks' cameras (8 bytes a block), met by its offset from the first block's
/// cameras. The original does not check the offset against the table: a camera step from
/// camera 0 reads the byte before it, the last byte of the replay block read (`block`, which
/// ends where the cameras begin). Any other offset past the table is refused.
#[derive(Clone, Copy)]
pub struct CameraByte(u32);

/// The byte `offset` on from the first block's cameras.
pub fn camera_byte(offset: u32) -> CameraByte {
    CameraByte(offset)
}

impl CameraByte {
    /// The byte `n` blocks on (8 bytes each).
    pub fn blocks_on(self, n: u32) -> CameraByte {
        CameraByte(self.0.wrapping_add(n.wrapping_mul(8)))
    }

    pub fn read(self, what: &'static str) -> Result<u8, BadInput> {
        let buffers = &crate::race::viewer_buffers_store::VIEWER_BUFFERS;
        match self.0 {
            u32::MAX => Ok(buffers.block[BLOCK_LAST].get().to_le_bytes()[3]),
            n if (n as usize) < TABLE_LEN => Ok(buffers.block_cameras.get(n as usize)),
            _ => Err(BadInput(what)),
        }
    }

    pub fn write(self, value: u8, what: &'static str) -> Result<(), BadInput> {
        let buffers = &crate::race::viewer_buffers_store::VIEWER_BUFFERS;
        match self.0 {
            u32::MAX => {
                let mut bytes = buffers.block[BLOCK_LAST].get().to_le_bytes();
                bytes[3] = value;
                buffers.block[BLOCK_LAST].set(u32::from_le_bytes(bytes));
                Ok(())
            }
            n if (n as usize) < TABLE_LEN => {
                buffers.block_cameras.set(n as usize, value);
                Ok(())
            }
            _ => Err(BadInput(what)),
        }
    }
}

/// The last dword of the replay block read.
const BLOCK_LAST: usize = 0xA03;

/// The byte `offset` on from the first block's cameras (8 bytes a block; the original does not
/// check the offset).
pub fn camera_byte_at(offset: usize) -> Result<u8, BadInput> {
    camera_byte(offset as u32).read("a replay camera byte")
}

// ---- the list menu (`menu::list_*`) ----

/// Whether `n` cells from `at` lie in `table`.
fn inside(table: &dyn Cells, at: usize, n: usize) -> Result<(), BadInput> {
    match at.checked_add(n) {
        Some(end) if end <= table.count() => Ok(()),
        _ => Err(BadInput("past a replay table")),
    }
}

/// `rep movsd` of `dwords` from `from`'s cell `from_at` on to `to`'s cell `to_at` on,
/// forwards; past either table refused before anything is written.
pub fn move_cells(
    to: &dyn Cells,
    to_at: usize,
    from: &dyn Cells,
    from_at: usize,
    dwords: usize,
) -> Result<(), BadInput> {
    inside(from, from_at, dwords)?;
    inside(to, to_at, dwords)?;
    for n in 0..dwords {
        to.set_bits(to_at + n, from.bits(from_at + n));
    }
    Ok(())
}

/// `std; rep movsd`: the same move from the last dword down (an overlapping move up).
pub fn move_cells_back(
    table: &dyn Cells,
    to_at: usize,
    from_at: usize,
    dwords: usize,
) -> Result<(), BadInput> {
    inside(table, from_at, dwords)?;
    inside(table, to_at, dwords)?;
    for n in (0..dwords).rev() {
        table.set_bits(to_at + n, table.bits(from_at + n));
    }
    Ok(())
}

/// `dwords` cells of `table` from `at` on set to 0.
pub fn clear_cells(table: &dyn Cells, at: usize, dwords: usize) -> Result<(), BadInput> {
    inside(table, at, dwords)?;
    for n in 0..dwords {
        table.set_bits(at + n, 0);
    }
    Ok(())
}

// ---- for `net::sort_players` ----
