//! The viewer's two plays, sharing one tail:
//! - `race::play_marked`: from the first mark (`mark_in`, frame 0) to the
//!   second (`mark_out`).
//! - `race::play_from_here`: from the block and frame shown (`replay_block`,
//!   `replay_frame`) to the end (the index's `block_count`); it jumps into `play_marked`'s tail.
//!
//! The tail: the block to stop at into `play_end`, `viewer_stopped` 0 (playing), the
//! start read (`drive::read_replay_frame` with the block and the frame; its failure not
//! looked at), the play (`race::play_replay_range`), `play_from_here` 0.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::race::viewer_flow::Play;

/// The shared tail: from block `play.from`, frame `play.frame`, to block
/// `play.end`. The start read's carry is not looked at.
pub(super) fn play_tail(
    race: &Race,
    o: crate::owners::Owners<'static>,
    play: Play,
) -> Result<(), Refused> {
    let replay = &race.replay;
    replay.replay_view().play_end.set(play.end);
    race.shared.replay_file().viewer_stopped.set(false);
    { crate::drive::read_replay_frame::read_replay_frame(race, o.system, play.from, play.frame) }?;
    crate::race::play_replay_range::play_replay_range(race, o)?;
    replay.replay_view().play_from_here.set(0);
    Ok(())
}

/// `play_marked`.
pub(super) fn play_marked(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::viewer_flow::marked_play`.
    play_tail(
        race,
        o,
        rc2k_core::race::viewer_flow::marked_play(
            race.viewer_buffers().mark_in.get(),
            race.viewer_buffers().mark_out.get(),
        ),
    )
}

/// `play_from_here`.
pub fn play_from_here(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let replay = &race.replay;
    // Logic in `rc2k_core::race::viewer_flow::play_from_here`.
    play_tail(
        race,
        o,
        rc2k_core::race::viewer_flow::play_from_here(
            replay.replay_view().replay_block.get(),
            replay.replay_view().replay_frame.get(),
            race.shared.replay_file().index.block_count.get(),
        ),
    )
}
