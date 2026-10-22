//! `audio::stop_all_voices` (0x40AFD9): command 7: every playing voice to the stopping list.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The playing list, if not empty, is put in front of the stopping list (its last voice's
//! `next` the stopping list's head) and emptied, and `g_channelsUsed` is 0. The channels
//! keep their `used` marks: the turn's update frees them as it stops the voices.
//! Refuses a playing list that does not end within 0x200 voices.
//! Compared in shadow mode. Self-test: a redirected copy on lists of the test's, 40 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

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
