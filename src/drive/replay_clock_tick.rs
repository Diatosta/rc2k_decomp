//! The replay's clock: `replay_frame` + 1, at 100 back to 0
//! with the frame `replay_block` + 1.

use crate::race::owner_replay::Replay;
use rc2k_core::drive::clocks::ReplayClock;

pub fn replay_clock_tick(replay: &Replay) {
    let was = ReplayClock {
        frame: replay.replay_view().replay_block.get(),
        hundredths: replay.replay_view().replay_frame.get(),
    };
    let mut c = was;
    c.tick();
    replay.replay_view().replay_frame.set(c.hundredths);
    if c.frame != was.frame {
        replay.replay_view().replay_block.set(c.frame);
    }
}
