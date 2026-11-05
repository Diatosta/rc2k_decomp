//! `race::spread_camera` (0x41E6BD): a block's camera byte changed (ebx, the old value in
//! dh): the new value (`[ebx]`) also into the next blocks' same byte (8 bytes on each)
//! while they held the old one, for the blocks after the one shown
//! (`dword_5BECD0 - dword_5C2530` less one at most).
//!
//! In:  ebx the byte, dh its old value
//! Out: dl the new value, eax the blocks gone over (from 1), ecx what was left of the
//!      count; no flags read
//!
//! Callers: `race::viewer_action` (four), `race::next_camera`, `race::previous_camera`, with
//! the block shown not the last. The count is not checked: 0 (or less) is limited only by the
//! bytes met; refused when the walk would pass the 0x708 blocks of the table, before anything
//! is written.

use crate::errors::Refused;
use crate::pointers::GamePtr;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::trap::Trap;
use crate::race::viewer_state_store::REPLAY_VIEW;

/// The Rust version: `byte` the changed camera byte, `old` its old value. What
/// the original leaves (dl, eax, ecx) its callers do not read: `viewer_action` and the
/// camera steps end with it, and `viewer_action` keeps every register.
pub fn spread_camera(byte: GamePtr, old: u8) -> Result<(), Refused> {
    // The camera bytes of the following blocks, 8 bytes apart (a stride over bytes).
    let at = |n: u32| byte.offset(n.wrapping_mul(8));
    let dl = byte.read::<u8>("a replay camera byte")?;
    let count = REPLAY_FILE
        .index_block_count
        .get()
        .wrapping_sub(REPLAY_VIEW.replay_block.get());
    // Converted: `rc2k_core::race::viewer::spread` (how far it goes, first).
    let trap = Trap::new();
    let s = rc2k_core::race::viewer::spread(count, old, |n| {
        trap.read_at::<u8>(at(n), "a replay camera byte")
    })?;
    trap.finish()?;
    for n in 1..=s.taken {
        at(n).write(dl, "a replay camera byte")?;
    }
    Ok(())
}
