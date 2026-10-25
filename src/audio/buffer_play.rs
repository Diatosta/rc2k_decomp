//! What `audio::is_playing`, `rewind_buffer`, `play_buffer`, `stop_buffer` and
//! `commit_3d` share: a voice's buffer started, stopped and asked after, one COM call each
//! (`audio/com.rs`).

use crate::audio::buffer_settings::a3d;
use crate::audio::store::AUDIO;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

pub const GET_STATUS: u32 = 0x24;
pub const PLAY: u32 = 0x30;
pub const SET_CURRENT_POSITION: u32 = 0x34;
pub const STOP: u32 = 0x48;
pub const COMMIT: u32 = 0x44;

/// `GetStatus`: the buffer plays.
pub const PLAYING: u32 = 1;

fn buffer(v: &Voice, a3d_name: &'static str) -> Result<u32, Refused> {
    if a3d() {
        return Err(BadInput(a3d_name).into());
    }
    if v.buffer == 0 {
        return Err(BadInput("a voice without its buffer").into());
    }
    Ok(v.buffer)
}

/// `GetStatus` (+0x24) into a dword: the status.
pub unsafe fn is_playing(v: &Voice) -> Result<u32, Refused> {
    let b = buffer(v, "A3D (`sub_40CE81`)")?;
    let mut status = 0u32;
    unsafe {
        let f: unsafe extern "system" fn(u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(b, GET_STATUS));
        f(b, &mut status as *mut u32 as u32);
    }
    Ok(status)
}

/// `SetCurrentPosition` (+0x34) with 0.
pub unsafe fn rewind_buffer(v: &Voice) -> Result<i32, Refused> {
    let b = buffer(v, "A3D (`sub_40CE96`)")?;
    Ok(unsafe { crate::audio::com::call1(b, SET_CURRENT_POSITION, 0) })
}

/// `Play` (+0x30) with 0, 0 and the looping flag for a looping voice (kind 1). Failing: the
/// carry, and `DSERR_BUFFERLOST` sets the lost flag. Returns eax and the carry.
pub unsafe fn play_buffer_in_game(v: &Voice) -> Result<(i32, bool), Refused> {
    let b = buffer(v, "A3D (`sub_40CEA0`)")?;
    // Converted: `rc2k_core::audio::buffers::{play_flags, lost}`.
    let flags = rc2k_core::audio::buffers::play_flags(v);
    let r = unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(b, PLAY));
        f(b, 0, 0, flags)
    };
    if r != 0 && rc2k_core::audio::buffers::lost(r) {
        AUDIO.sound_buffer_lost.set(1);
    }
    Ok((r, r != 0))
}

/// `Stop` (+0x48).
pub unsafe fn stop_buffer(v: &Voice) -> Result<i32, Refused> {
    let b = buffer(v, "A3D (`sub_40CEBB`)")?;
    Ok(unsafe { crate::audio::com::call(b, STOP) })
}

/// The 3D listener's `CommitDeferredSettings` (+0x44).
pub unsafe fn commit_3d() -> Result<i32, Refused> {
    let object = AUDIO.sound_listener.get();
    if object == 0 {
        return Err(BadInput("no 3D listener").into());
    }
    Ok(unsafe { crate::audio::com::call(object, COMMIT) })
}
