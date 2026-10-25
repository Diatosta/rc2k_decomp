//! The viewer plays a replay from `var\tmprpl` (the last race's, or a saved one copied
//! there first), block by block (`dword_5C2530`, 0 .. `dword_5BECD0`; 100 frames a block,
//! the frame in `dword_5C2534`), and edits it: two marks (`dword_96E478`, `dword_96E47C`),
//! cut, copy and paste of blocks (the clipboard `unk_96E488`, `unk_971CC8`), one undo
//! (`dword_9673F4` ..), the cameras of each block (`unk_963BB4`: 8 bytes a block, the
//! first 8 of the block's data), and saving the result as a replay slot.
//!
//! The tests: a copy of the original whose calls go to `install::stand_ins` (`copy`),
//! noted by `note` with eax, ebx, ecx and edx and answered by the test's `answer` (eax and
//! the flags, memory written); the Rust side calls `test_part`, which does the same. Then
//! `run_both` compares the registers, the flags the callers read, the calls and memory.

use crate::physics::flow::{a, g, p};

// ---- the replay being viewed ----

pub const LAST_RACE: u32 = 0xDE; // the slot of `var\tmprpl` as it is
pub const DEVELOPER_KEYS: usize = 0x5C2CEB; // dword_5C2CEB
pub const PICTURE_RATE: usize = 0x5C2CEF; // dword_5C2CEF: pictures a second, 1 .. 100
pub const PICTURE_TIME: usize = 0x5C2CF3; // dword_5C2CF3: ms a picture
pub const CAMERA_NAMES: usize = 0x5C291C; // word_5C291C: the cameras' text ids, 0x2F
/// A variable of the image's zeroed tail (`race::store::buffers`).
type Buffer = crate::race::store::buffers::Slot;

pub const BLOCK_DATA: usize = 0x9613A4; // dword_9613A4: the block read, 0x2810 bytes
pub const BLOCK_CURSOR: Buffer = Buffer::at(0x9613A0); // dword_9613A0
pub const CAMERAS: usize = 0x963BB4; // unk_963BB4: 8 bytes a block
pub const UNDO_BLOCKS: Buffer = Buffer::at(0x9673F4); // dword_9673F4: 0 = nothing to undo
pub const UNDO_PLACES: usize = 0x9673F8; // dword_9673F8: 0x3840 bytes
pub const UNDO_CAMERAS: usize = 0x96AC38; // unk_96AC38: 0x3840 bytes
pub const TABLE_LEN: usize = 0x3840; // 0x708 blocks of 8 bytes
pub const MARK_IN: usize = 0x96E478; // dword_96E478
pub const MARK_OUT: usize = 0x96E47C; // dword_96E47C
pub const CAMERA_EDIT: Buffer = Buffer::at(0x96E480); // dword_96E480: 0, or the camera (1, 2) edited
pub const CLIP_BLOCKS: Buffer = Buffer::at(0x96E484); // dword_96E484: blocks on the clipboard
pub const CLIP_PLACES: usize = 0x96E488; // unk_96E488
pub const CLIP_CAMERAS: usize = 0x971CC8; // unk_971CC8
pub const HEADER: usize = 0x528504; // dword_528504: the file's header, 0x430

// ---- the list menu (`menu::list_*`) ----

pub fn gb(x: impl crate::store::At) -> u8 {
    unsafe { (a(x) as *const u8).read() }
}

/// `rep movsd` of `dwords` from `from` to `to` (absolute addresses), forwards.
pub fn move_dwords(to: usize, from: usize, dwords: usize) {
    unsafe {
        let (to, from) = (to as *mut u32, from as *const u32);
        for n in 0..dwords {
            to.add(n).write_unaligned(from.add(n).read_unaligned());
        }
    }
}

/// A table of `dwords` dwords at the absolute address `at` (dword-aligned) as a slice, for
/// the core.
///
/// # Safety
/// No other reference to the table may be alive while the slice is.
pub unsafe fn table(at: usize, dwords: usize) -> &'static mut [u32] {
    let at = at as *mut u32;
    debug_assert!(at.is_aligned());
    unsafe { std::slice::from_raw_parts_mut(at, dwords) }
}

// ---- for `net::sort_players` ----

/// `g` and `p` for other modules.
pub fn get(x: impl crate::store::At) -> u32 {
    g(x)
}

pub fn put(x: impl crate::store::At, v: u32) {
    p(x, v)
}
