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
//! Out: eax the block played from, ebx and ecx its frame; no flags read. Their one caller
//!      (`sub_41DE81`) reads only eax after them (taint at 0x41E1FF, 0x41E215, 0x41E25F:
//!      0x41DE7D), which the Rust `viewer_action` does not need (its ZF is whether the
//!      action was one).
//!
//! Callers: `race::viewer_action` (0x1A, 0x25 .. 0x28 `play_marked`; 6 and 0x22, 0x23
//! `play_from_here`).

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::race::viewer_flow::Play;

/// The shared tail (0x41E231): from block `play.from`, frame `play.frame`, to block
/// `play.end`. The start read's carry is not looked at.
pub fn play_tail(race: &Race, play: Play) -> Result<(), Refused> {
    let input = crate::input::owner::input();
    let sound = crate::sound::owner::sound();
    let replay = &race.replay;
    replay.replay_view.play_end.set(play.end);
    race.shared.replay_file.viewer_stopped.set(0);
    { crate::drive::read_replay_frame::read_replay_frame(race, play.from, play.frame) }?;
    crate::race::play_replay_range::play_replay_range(race, sound, input)?;
    replay.replay_view.play_from_here.set(0);
    Ok(())
}

/// `play_marked`.
pub fn play_marked(race: &Race) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::marked_play`.
    play_tail(
        race,
        rc2k_core::race::viewer_flow::marked_play(
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .mark_in
                .get(),
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .mark_out
                .get(),
        ),
    )
}

/// `play_from_here`.
pub fn play_from_here(race: &Race) -> Result<(), Refused> {
    let replay = &race.replay;
    // Converted: `rc2k_core::race::viewer_flow::play_from_here`.
    play_tail(
        race,
        rc2k_core::race::viewer_flow::play_from_here(
            replay.replay_view.replay_block.get(),
            replay.replay_view.replay_frame.get(),
            race.shared.replay_file.index_block_count.get(),
        ),
    )
}
