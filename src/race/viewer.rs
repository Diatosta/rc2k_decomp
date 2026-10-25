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

// ---- the replay being viewed ----

pub const LAST_RACE: u32 = 0xDE; // the slot of `var\tmprpl` as it is
pub const TABLE_LEN: usize = 0x3840; // 0x708 blocks of 8 bytes
/// The byte `offset` on from the first block's cameras (8 bytes a block; the original does not
/// check the offset).
pub fn camera_byte_at(offset: usize) -> u8 {
    let at = crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .block_cameras
        .addr()
        .wrapping_add(offset);
    unsafe { (at as *const u8).read() }
}

// ---- the list menu (`menu::list_*`) ----

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
