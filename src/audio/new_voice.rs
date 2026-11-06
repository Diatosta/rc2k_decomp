//! `audio::new_voice` (0x40B04C): the next voice of the pool.
//!
//! In:  nothing
//! Out: esi = the voice; everything else preserved
//!
//! `g_voices[g_voiceCount]`, and the count + 1; with 0x200 in use fatal error 0x80A28501.
//! One caller, command 3 (`sub_40AEB9`).

use crate::audio::store::AudioState;
use crate::errors::{FatalError, Refused};

pub fn new_voice_in_game(state: &AudioState) -> Result<u32, Refused> {
    // Converted: `rc2k_core::audio::voices::new_voice`.
    let mut n = state.voice_count.get();
    let Some(place) = rc2k_core::audio::voices::new_voice(&mut n) else {
        return Err(FatalError {
            code: rc2k_core::audio::voices::POOL_USED_UP,
            message: "",
        }
        .into());
    };
    state.voice_count.set(n);
    // Its id (the original: the place's address, no check against the pool).
    Ok(crate::audio::voices::voice_id(place))
}
