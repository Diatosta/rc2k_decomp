//! The viewer's two plays (IDA 2026-10-19), sharing one tail (0x41E231):
//! - `race::play_marked` (0x41E224): from the first mark (`dword_96E478`, frame 0) to the
//!   second (`dword_96E47C`).
//! - `race::play_from_here` (0x41E1E2): from the block and frame shown (`dword_5C2530`,
//!   `dword_5C2534`) to the end (`dword_5BECD0`); it jumps into `play_marked`'s tail.
//!
//! The tail: the block to stop at into `dword_5C2538`, `dword_5BEC98` 0 (playing), the
//! start read (`drive::read_replay_frame`, eax the block, ebx the frame; its carry not
//! looked at), the play (`race::play_replay_range`), `dword_5C253C` 0.
//!
//! In:  nothing
//! Out: eax the block played from, ebx and ecx its frame; no flags read. Their one caller
//!      (`sub_41DE81`) reads only eax after them (taint at 0x41E1FF, 0x41E215, 0x41E25F:
//!      0x41DE7D), which the Rust `viewer_action` does not need (its ZF is whether the
//!      action was one).
//!
//! Callers: `race::viewer_action` (0x1A, 0x25 .. 0x28 `play_marked`; 6 and 0x22, 0x23
//! `play_from_here`).

use crate::errors::Refused;
use crate::race::viewer::{
    BLOCK, BLOCKS, FRAME, MARK_IN, MARK_OUT, PLAY_END, PLAY_FROM_HERE, STOPPED, get, put,
};
use rc2k_core::race::viewer_flow::Play;

/// The shared tail (0x41E231): from block `play.from`, frame `play.frame`, to block
/// `play.end`. The start read's carry is not looked at.
pub fn play_tail(play: Play) -> Result<(), Refused> {
    put(PLAY_END, play.end);
    put(STOPPED, 0);
    unsafe { crate::drive::read_replay_frame::read_replay_frame(play.from, play.frame) }?;
    crate::race::play_replay_range::play_replay_range()?;
    put(PLAY_FROM_HERE, 0);
    Ok(())
}

/// `play_marked`.
pub fn play_marked() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::marked_play`.
    play_tail(rc2k_core::race::viewer_flow::marked_play(
        get(MARK_IN),
        get(MARK_OUT),
    ))
}

/// `play_from_here`.
pub fn play_from_here() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::play_from_here`.
    play_tail(rc2k_core::race::viewer_flow::play_from_here(
        get(BLOCK),
        get(FRAME),
        get(BLOCKS),
    ))
}
