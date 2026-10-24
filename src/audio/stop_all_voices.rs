//! `audio::stop_all_voices` (0x40AFD9): command 7: every playing voice to the stopping list.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The playing list, if not empty, is put in front of the stopping list (its last voice's
//! `next` the stopping list's head) and emptied, and `g_channelsUsed` is 0. The channels
//! keep their `used` marks: the turn's update frees them as it stops the voices.
//! Refuses a playing list that does not end within 0x200 voices.

use crate::audio::voices::*;
use crate::errors::Refused;

// ---- Rust version ----

/// `playing`, `stopping`, `used`: where the two lists' heads and the count are.
pub unsafe fn stop_all_voices(playing: u32, stopping: u32, used: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::voices::stop_all`.
    let w = World {
        playing,
        stopping,
        channels_used: used,
        ..World::game()
    };
    unsafe {
        let mut l = lists(&w);
        rc2k_core::audio::voices::stop_all(&mut GamePool::new(&w), &mut l)?;
        store_lists(&w, &l);
    }
    Ok(())
}
