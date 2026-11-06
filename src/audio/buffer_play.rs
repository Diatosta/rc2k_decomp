//! What `audio::is_playing`, `rewind_buffer`, `play_buffer`, `stop_buffer` and
//! `commit_3d` share: a voice's buffer started, stopped and asked after, one COM call each
//! (`audio/com.rs`).

use crate::audio::buffer_settings::a3d;
use crate::audio::store::AudioState;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

/// `GetStatus`: the buffer plays.
pub const PLAYING: u32 = 1;

fn buffer(state: &AudioState, v: &Voice, a3d_name: &'static str) -> Result<u32, Refused> {
    if a3d(state) {
        return Err(BadInput(a3d_name).into());
    }
    if v.buffer == 0 {
        return Err(BadInput("a voice without its buffer").into());
    }
    Ok(v.buffer)
}

/// `GetStatus` (+0x24) into a dword: the status.
pub fn is_playing(state: &AudioState, v: &Voice) -> Result<u32, Refused> {
    let b = buffer(state, v, "A3D (`sub_40CE81`)")?;
    // A failed call leaves the status 0.
    Ok(crate::platform_sound::status(b).unwrap_or(0))
}

/// `SetCurrentPosition` (+0x34) with 0.
pub fn rewind_buffer(state: &AudioState, v: &Voice) -> Result<i32, Refused> {
    let b = buffer(state, v, "A3D (`sub_40CE96`)")?;
    Ok(crate::platform_sound::set_current_position(b, 0))
}

/// `Play` (+0x30) with 0, 0 and the looping flag for a looping voice (kind 1). Failing: the
/// carry, and `DSERR_BUFFERLOST` sets the lost flag. Returns eax and the carry.
pub fn play_buffer_in_game(state: &AudioState, v: &Voice) -> Result<(i32, bool), Refused> {
    let b = buffer(state, v, "A3D (`sub_40CEA0`)")?;
    // Converted: `rc2k_core::audio::buffers::{play_flags, lost}`.
    let flags = rc2k_core::audio::buffers::play_flags(v);
    let r = crate::platform_sound::play(b, flags);
    if r != 0 && rc2k_core::audio::buffers::lost(r) {
        state.sound_buffer_lost.set(1);
    }
    Ok((r, r != 0))
}

/// `Stop` (+0x48).
pub fn stop_buffer(state: &AudioState, v: &Voice) -> Result<i32, Refused> {
    let b = buffer(state, v, "A3D (`sub_40CEBB`)")?;
    Ok(crate::platform_sound::stop(b))
}

/// The 3D listener's `CommitDeferredSettings` (+0x44).
pub fn commit_3d(state: &AudioState) -> Result<i32, Refused> {
    let object = state.sound_listener.get();
    if object == 0 {
        return Err(BadInput("no 3D listener").into());
    }
    Ok(crate::platform_sound::commit_deferred(object))
}
