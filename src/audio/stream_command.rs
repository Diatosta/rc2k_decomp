//! `audio::stream_command` (0x40C194): command 9, the sound stream (`g_soundStream`).
//!
//! In:  eax = the file's path, ecx = the volume, edx = the action
//! Out: every register preserved
//!
//! See `audio/stream_commands.rs`. Callers: `audio::take_commands` (command 9), the
//! thread's turn (action 1), `audio::stop_streams`. Shadow mode runs the original alone.
//! Self-test: a redirected copy whose player is the test's and whose four calls go to
//! stand-ins (`stream_commands::handler`), 180 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::rva;
use crate::install::stand_ins::stand_in;

/// The leaves' places by their addresses (and `files::file_exists`).
pub fn stand_in_for(callee: usize) -> Option<usize> {
    [0x40C1EA, 0x40C349, 0x40C468, 0x40C4B2, 0x402EBE]
        .iter()
        .position(|&t| rva(t) == callee)
        .map(|n| stand_in(n as u32))
}
