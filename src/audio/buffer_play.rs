//! What `audio::is_playing`, `rewind_buffer`, `play_buffer`, `stop_buffer` and `commit_3d` share:
//! a voice's buffer started, stopped and asked after, one COM call each.

use crate::audio::buffer_settings::a3d;
use crate::audio::own::AudioOwn;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

/// `GetStatus`: the buffer plays.
pub const PLAYING: u32 = 1;

fn buffer(state: &AudioOwn, v: &Voice, a3d_name: &'static str) -> Result<u32, Refused> {
    if a3d(state) {
        return Err(BadInput(a3d_name).into());
    }
    if v.buffer == 0 {
        return Err(BadInput("a voice without its buffer").into());
    }
    Ok(v.buffer)
}

/// `GetStatus` into a dword: the status.
pub fn is_playing(state: &AudioOwn, v: &Voice) -> Result<u32, Refused> {
    let b = buffer(state, v, "A3D (0x40CE81)")?;
    // A failed call leaves the status 0.
    Ok(crate::platform::sound::status(b).unwrap_or(0))
}

/// `SetCurrentPosition` with 0.
pub fn rewind_buffer(state: &AudioOwn, v: &Voice) -> Result<i32, Refused> {
    let b = buffer(state, v, "A3D (0x40CE96)")?;
    Ok(crate::platform::sound::set_current_position(b, 0))
}

/// `Play` with 0, 0 and the looping flag for a looping voice (kind 1). A lost buffer sets the lost
/// flag. Returns the result code and whether it failed.
pub fn play_buffer(state: &AudioOwn, v: &Voice) -> Result<(i32, bool), Refused> {
    let b = buffer(state, v, "A3D (0x40CEA0)")?;
    let flags = rc2k_core::audio::buffers::play_flags(v);
    let r = crate::platform::sound::play(b, flags);
    if r != 0 && rc2k_core::audio::buffers::lost(r) {
        state.sound_buffer_lost.set(1);
    }
    Ok((r, r != 0))
}

/// `Stop`.
pub fn stop_buffer(state: &AudioOwn, v: &Voice) -> Result<i32, Refused> {
    let b = buffer(state, v, "A3D (0x40CEBB)")?;
    Ok(crate::platform::sound::stop(b))
}

/// The 3D listener's `CommitDeferredSettings`.
pub fn commit_3d(state: &AudioOwn) -> Result<i32, Refused> {
    let object = state.sound_listener.get();
    if object == 0 {
        return Err(BadInput("no 3D listener").into());
    }
    Ok(crate::platform::sound::commit_deferred(object))
}
