//! The viewer's two plays, sharing one tail (0x41E231):
//! - `race::play_marked` (0x41E224): from the first mark (`mark_in`, frame 0) to the
//!   second (`mark_out`).
//! - `race::play_from_here` (0x41E1E2): from the block and frame shown (`replay_block`,
//!   `replay_frame`) to the end (`index_block_count`); it jumps into `play_marked`'s tail.
//!
//! The tail: the block to stop at into `play_end`, `viewer_stopped` 0 (playing), the
//! start read (`drive::read_replay_frame` with the block and the frame; its failure not
//! looked at), the play (`race::play_replay_range`), `play_from_here` 0.
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
