//! `drive::replay_clock_tick` (0x460C20): the replay's clock: `dword_5C2534` + 1, at 100
//! back to 0 with the frame `dword_5C2530` + 1.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::replay_step`.

use crate::race::viewer_state_store::REPLAY_VIEW;
use rc2k_core::drive::clocks::ReplayClock;

/// The Rust version.
// Converted: `rc2k_core::drive::clocks::ReplayClock::tick`.
pub fn replay_clock_tick() {
    let was = ReplayClock {
        frame: REPLAY_VIEW.replay_block.get(),
        hundredths: REPLAY_VIEW.replay_frame.get(),
    };
    let mut c = was;
    c.tick();
    REPLAY_VIEW.replay_frame.set(c.hundredths);
    if c.frame != was.frame {
        REPLAY_VIEW.replay_block.set(c.frame);
    }
}
