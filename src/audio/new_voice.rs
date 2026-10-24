//! `audio::new_voice` (0x40B04C): the next voice of the pool.
//!
//! In:  nothing
//! Out: esi = the voice; everything else preserved
//!
//! `g_voices[g_voiceCount]`, and the count + 1; with 0x200 in use fatal error 0x80A28501.
//! One caller, command 3 (`sub_40AEB9`).
//! Compared in shadow mode. Self-test: a redirected copy on a pool of the test's, 5 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::voices::*;
use crate::errors::{FatalError, Refused};
use crate::store::game_addr;

/// `count`: where the count is kept; `voices`: the pool.
pub unsafe fn new_voice(count: u32, voices: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::audio::voices::new_voice`.
    unsafe {
        let mut n = read(count);
        let Some(place) = rc2k_core::audio::voices::new_voice(&mut n) else {
            return Err(FatalError {
                code: rc2k_core::audio::voices::POOL_USED_UP,
                message: "",
            }
            .into());
        };
        write(count, n);
        Ok(voices + place * std::mem::size_of::<Voice>() as u32)
    }
}

pub unsafe fn new_voice_in_game() -> Result<u32, Refused> {
    unsafe {
        new_voice(
            game_addr(VOICE_COUNT_RVA) as u32,
            game_addr(VOICES_RVA) as u32,
        )
    }
}
