//! `drive::read_replay_frame` (0x41EBAB): a replay's frame eax (with the tenth ebx) read,
//! while a replay plays from its file (`dword_5BEC88` 1; else CF). Frames past 0x708 and
//! tenths past 100 are the fatal errors 0x80A61151 and 0x80A61152 (raised as the
//! original). At or past the last frame (`dword_5BECD0`): the last, `dword_5BEC90` 1, the
//! tenth 0. Else the tenth into `dword_5C2534`, `dword_5BEC90` 0, the frame into
//! `dword_5C2530`, the file (`h`) put at the frame's place (`lDistanceToMove[frame]`,
//! `files::seek`) and its bytes (`dword_5BECDC[frame]`, packed when below 0) read into
//! `dword_9613A4` (`drive::read_packed`). All in `pusha` / `popa`.
//!
//! In:  eax the frame, ebx the tenth
//! Out: CF set when not playing or the seek or read failed; every register kept
//!
//! Callers: `drive::read_replay_frame_0`, `sub_41DE81` (four), `sub_41E224`. Refuses what
//! `read_packed` refuses.

use crate::errors::{FatalError, Refused};
use crate::race::replay_file_store::REPLAY_FILE;
use rc2k_core::drive::replay_frames::{frame_fault, past_end};

/// The Rust version: CF.
pub fn read_replay_frame(frame: u32, tenth: u32) -> Result<bool, Refused> {
    if REPLAY_FILE.replay_open.get() != 1 {
        return Ok(true);
    }
    if let Some(code) = frame_fault(frame, tenth) {
        return Err(FatalError { code, message: "" }.into());
    }
    if past_end(frame, REPLAY_FILE.index_block_count.get()) {
        REPLAY_FILE.replay_at_end.set(1);
        crate::race::owner::race()
            .replay
            .replay_view
            .replay_block
            .set(REPLAY_FILE.index_block_count.get());
        crate::race::owner::race()
            .replay
            .replay_view
            .replay_frame
            .set(0);
        return Ok(false);
    }
    crate::race::owner::race()
        .replay
        .replay_view
        .replay_frame
        .set(tenth);
    REPLAY_FILE.replay_at_end.set(0);
    crate::race::owner::race()
        .replay
        .replay_view
        .replay_block
        .set(frame);
    let place = crate::race::replay::block_place(2 * frame as usize)?; // (place, count) a frame
    if { crate::files::handle::seek(REPLAY_FILE.replay_handle.get(), place as i32, 0) }.is_none() {
        return Ok(true);
    }
    let count = crate::race::replay::block_place(2 * frame as usize + 1)?;
    let (_, carry, _) =
        { crate::drive::read_packed::read_packed(REPLAY_FILE.replay_handle.get(), count) }?;
    Ok(carry)
}
