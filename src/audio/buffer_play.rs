//! What `audio::is_playing`, `rewind_buffer`, `play_buffer`, `stop_buffer` and `commit_3d` share:
//! a voice's buffer started, stopped and asked after, one COM call each.

use crate::audio::own::AudioOwn;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

/// `GetStatus`: the buffer plays.
pub const PLAYING: u32 = 1;

fn buffer(v: &Voice) -> Result<u32, Refused> {
    if v.buffer == 0 {
        return Err(BadInput("a voice without its buffer").into());
    }
    Ok(v.buffer)
}

/// A play that failed goes to the program's log: the sound it was for stays silent.
pub fn log_failed_play(what: &str, result: i32) {
    crate::logfile::program_log::log(&format!(
        "audio: playing {what} failed (result {:#x})",
        result as u32
    ));
}

/// `GetStatus` into a dword: the status.
pub fn is_playing(v: &Voice) -> Result<u32, Refused> {
    let b = buffer(v)?;
    // A failed call leaves the status 0.
    Ok(crate::platform::sound::status(b).unwrap_or(0))
}

/// `SetCurrentPosition` with 0.
pub fn rewind_buffer(v: &Voice) -> Result<i32, Refused> {
    let b = buffer(v)?;
    Ok(crate::platform::sound::set_current_position(b, 0))
}

/// `Play` with 0, 0 and the looping flag for a looping voice (kind 1). A failure is logged.
/// Returns the result code and whether it failed.
pub fn play_buffer(v: &Voice) -> Result<(i32, bool), Refused> {
    let b = buffer(v)?;
    let flags = rc2k_core::audio::buffers::play_flags(v);
    let r = crate::platform::sound::play(b, flags);
    if r != 0 {
        log_failed_play("a voice's buffer", r);
    }
    Ok((r, r != 0))
}

/// `Stop`.
pub fn stop_buffer(v: &Voice) -> Result<i32, Refused> {
    let b = buffer(v)?;
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
