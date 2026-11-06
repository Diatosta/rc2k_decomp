//! `drive::replay_clock_tick` (0x460C20): the replay's clock: `dword_5C2534` + 1, at 100
//! back to 0 with the frame `dword_5C2530` + 1.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::replay_step`.

use crate::race::owner_replay::Replay;
use rc2k_core::drive::clocks::ReplayClock;

/// The Rust version.
// Converted: `rc2k_core::drive::clocks::ReplayClock::tick`.
pub fn replay_clock_tick(replay: &Replay) {
    let was = ReplayClock {
        frame: replay.replay_view.replay_block.get(),
        hundredths: replay.replay_view.replay_frame.get(),
    };
    let mut c = was;
    c.tick();
    replay.replay_view.replay_frame.set(c.hundredths);
    if c.frame != was.frame {
        replay.replay_view.replay_block.set(c.frame);
    }
}
