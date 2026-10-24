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

pub const OPEN: usize = 0x5BEC88; // dword_5BEC88: `h` is the replay's file
pub const HANDLE: usize = 0x5BEC8C; // h
pub const AT_END: usize = 0x5BEC90; // dword_5BEC90: the last frame was reached
pub const HOLD_REVS: usize = 0x5BEC94; // g_holdCarRevs
pub const STOPPED: usize = 0x5BEC98; // dword_5BEC98: 1 = not playing
pub const MODE: usize = 0x5BEC9C; // dword_5BEC9C: the viewer's eax
pub const SLOT: usize = 0x5BECA0; // dword_5BECA0: the replay (222 the last race's)
pub const SOUND: usize = 0x5BECA4; // dword_5BECA4: the menu sound to play
pub const EDITING: usize = 0x5BECA8; // dword_5BECA8: the editing keys work
pub const INDEX: usize = 0x5BECC8; // "RCIB": the index, 0x3850 bytes
pub const BLOCKS: usize = 0x5BECD0; // dword_5BECD0: blocks in the replay
pub const PLACES: usize = 0x5BECD8; // lDistanceToMove: (offset, size) a block
pub const LAST_RACE: u32 = 0xDE; // the slot of `var\tmprpl` as it is
pub const BLOCK: usize = 0x5C2530; // dword_5C2530: the block shown
pub const FRAME: usize = 0x5C2534; // dword_5C2534: its frame, 0 .. 99
pub const PLAY_END: usize = 0x5C2538; // dword_5C2538: playing stops before this block
pub const PLAY_FROM_HERE: usize = 0x5C253C; // dword_5C253C
pub const SNAPSHOT: usize = 0x5C2540; // dword_5C2540: a picture of the next frame asked
pub const LAST_RACE_VIEWED: usize = 0x5C2544; // dword_5C2544: the slot is 222
pub const ONLY_SHOWN: usize = 0x5C2548; // dword_5C2548: modes 4 and 5
pub const KEYS_IGNORED: usize = 0x5C254C; // dword_5C254C: mode 5
pub const KEY_AFTER_PLAY: usize = 0x5C2550; // dword_5C2550
pub const WAIT_RELEASE: usize = 0x5C2554; // dword_5C2554: keys held after playing
pub const ESCAPES: usize = 0x5C2558; // dword_5C2558: Escape twice leaves
pub const MENU: usize = 0x5C255C; // unk_5C255C: the viewer's list (items at +0x18)
pub const RECORDING: usize = 0x5C2CCC; // g_replayRecording: the frames saved as pictures
pub const PICTURES_STARTED: usize = 0x5C2CD4; // dword_5C2CD4
pub const PICTURES_STARTED_2: usize = 0x5C2CD0; // dword_5C2CD0
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
pub const CAR_HEADER: usize = 0x719B14; // dword_719B14: 0x400 bytes
pub const ONE_PLAYER: usize = 0x719B34; // dword_719B34 (the header's)
pub const FIRST_CAR: usize = 0x719B24; // dword_719B24 (the header's)

// ---- the list menu (`menu::list_*`) ----

pub const CHOICE: usize = 0x51D407; // dword_51D407: the item highlighted
pub const CHOICE_BEFORE: usize = 0x51D40B; // dword_51D40B
pub const REPEATS: usize = 0x51D40F; // dword_51D40F
pub const LAST_KEY: usize = 0x51D413; // byte_51D413
pub const LIST_SHOWN: usize = 0x51D415; // byte_51D415
pub const LIST_LEAVE: usize = 0x51D416; // byte_51D416: 1 leaves
pub const KEY_CLOCK: usize = 0x51D41B; // dword_51D41B
pub const FRAME_CLOCK: usize = 0x51D41F; // dword_51D41F
pub const MOVE_CLOCK: usize = 0x51D423; // dword_51D423

pub fn gb(x: impl crate::store::At) -> u8 {
    unsafe { (a(x) as *const u8).read() }
}

pub fn pb(x: impl crate::store::At, v: u8) {
    unsafe { (a(x) as *mut u8).write(v) }
}

/// `rep movsd` of `dwords` from `from` to `to` (IDA addresses), forwards.
pub fn move_dwords(to: usize, from: usize, dwords: usize) {
    unsafe {
        let (to, from) = (a(to) as *mut u32, a(from) as *const u32);
        for n in 0..dwords {
            to.add(n).write_unaligned(from.add(n).read_unaligned());
        }
    }
}

/// A table of `dwords` dwords at IDA address `x` (dword-aligned there, so here too) as a
/// slice, for the core.
///
/// # Safety
/// No other reference to the table may be alive while the slice is.
pub unsafe fn table(x: usize, dwords: usize) -> &'static mut [u32] {
    let at = a(x) as *mut u32;
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
