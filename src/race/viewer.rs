//! The viewer plays a replay from `var\tmprpl` (the last race's, or a saved one copied
//! there first), block by block (`dword_5C2530`, 0 .. `dword_5BECD0`; 100 frames a block,
//! the frame in `dword_5C2534`), and edits it: two marks (`dword_96E478`, `dword_96E47C`),
//! cut, copy and paste of blocks (the clipboard `unk_96E488`, `unk_971CC8`), one undo
//! (`dword_9673F4` ..), the cameras of each block (`unk_963BB4`: 8 bytes a block, the
//! first 8 of the block's data), and saving the result as a replay slot.
//!
//! The tests: a copy of the original whose calls go to `install::stand_ins` (`copy`), noted by
//! `note` with eax, ebx, ecx and edx and answered by the test's `answer`; the Rust side calls
//! `test_part`, which does the same.

use crate::errors::BadInput;
use crate::math::cells::Cells;
use crate::pointers::GamePtr;

// ---- the replay being viewed ----

pub const LAST_RACE: u32 = 0xDE; // the slot of `var\tmprpl` as it is
pub const TABLE_LEN: usize = 0x3840; // 0x708 blocks of 8 bytes
/// The byte `offset` on from the first block's cameras (8 bytes a block; the original does not
/// check the offset).
pub fn camera_byte_at(offset: usize) -> Result<u8, BadInput> {
    camera_ptr(offset as u32).read("a replay camera byte")
}

/// The place `offset` bytes on from the first block's cameras. Unbounded: the original does
/// not check the offset against the table (a camera step from camera 0 reads the byte
/// before it).
pub fn camera_ptr(offset: u32) -> GamePtr {
    GamePtr::to(&crate::race::viewer_buffers_store::VIEWER_BUFFERS.block_cameras)
        .unbounded()
        .offset(offset)
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
