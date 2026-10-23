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
//! Callers: `drive::read_replay_frame_0`, `sub_41DE81` (four), `sub_41E224`.
//! Refuses what `read_packed` refuses.
//! Self-test: the original and the Rust version with a test replay file (the temp
//! folder; `h` and the frame tables pointed at it, saved and put back): not playing,
//! frames plain and packed, the last frame.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{FatalError, Refused};
use crate::physics::flow::{a, g, p};
use rc2k_core::drive::replay_frames::{frame_fault, past_end};

/// `dword_9613A4`: the frame read.
pub const FRAME: usize = 0x9613A4;
const FILE: usize = 0x5BEC8C; // h
const PLACES: usize = 0x5BECD8; // lDistanceToMove: (place, count) a frame

/// The Rust version: CF.
pub unsafe fn read_replay_frame(frame: u32, tenth: u32) -> Result<bool, Refused> {
    if g(0x5BEC88) != 1 {
        return Ok(true);
    }
    if let Some(code) = frame_fault(frame, tenth) {
        return Err(FatalError { code, message: "" }.into());
    }
    if past_end(frame, g(0x5BECD0)) {
        p(0x5BEC90, 1);
        p(0x5C2530, g(0x5BECD0));
        p(0x5C2534, 0);
        return Ok(false);
    }
    p(0x5C2534, tenth);
    p(0x5BEC90, 0);
    p(0x5C2530, frame);
    let place = g(PLACES + 8 * frame as usize);
    if unsafe { crate::files::handle::seek(g(FILE), place as i32, 0) }.is_none() {
        return Ok(true);
    }
    let count = g(PLACES + 4 + 8 * frame as usize);
    let (_, carry, _) =
        unsafe { crate::drive::read_packed::read_packed(g(FILE), a(FRAME), count) }?;
    Ok(carry)
}
