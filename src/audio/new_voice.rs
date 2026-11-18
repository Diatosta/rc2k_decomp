//! The next voice of the pool and the count + 1; with `VOICES` in use fatal error.

use crate::audio::own::AudioOwn;
use crate::errors::{FatalError, Refused};

pub fn new_voice(state: &AudioOwn) -> Result<u32, Refused> {
    let mut n = state.voice_count.get();
    let Some(place) = rc2k_core::audio::voices::new_voice(&mut n) else {
        return Err(FatalError {
            code: rc2k_core::audio::voices::POOL_USED_UP,
            message: "",
        }
        .into());
    };
    state.voice_count.set(n);
    Ok(crate::audio::voices::voice_id(place))
}
