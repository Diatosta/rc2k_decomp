//! What `audio::set_buffer_volume`, `set_buffer_pan`, `set_buffer_frequency` and
//! `set_buffer_place` share: a voice's settings to its DirectSound buffer (one COM call
//! each).

use crate::audio::com;
use crate::audio::store::AUDIO;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

pub const SET_VOLUME: u32 = 0x3C;
pub const SET_PAN: u32 = 0x40;
pub const SET_FREQUENCY: u32 = 0x44;
pub const SET_POSITION: u32 = 0x4C;

pub fn a3d() -> bool {
    AUDIO.use_a3d.get() == 1
}

/// `g_volumeTable`'s entry `n`, for the core.
pub fn volumes(n: u32) -> u32 {
    AUDIO.volume_table[n as usize].get()
}

fn buffer(v: &Voice) -> Result<u32, Refused> {
    if v.buffer == 0 {
        return Err(BadInput("a voice without its buffer").into());
    }
    Ok(v.buffer)
}

/// `SetVolume` with `g_volumeTable[volume * volume_factor >> 6]` (signed). Returns eax.
pub unsafe fn set_buffer_volume(v: &Voice) -> Result<i32, Refused> {
    if a3d() {
        return Err(BadInput("A3D (`sub_40CEFE`)").into());
    }
    // Converted: `rc2k_core::audio::levels::buffer_volume`.
    let level = rc2k_core::audio::levels::buffer_volume(v.volume, v.volume_factor, volumes)?;
    let b = buffer(v)?;
    Ok(unsafe { com::call1(b, SET_VOLUME, level) })
}

/// `SetPan`, nothing in 3D: a pan below 0 is `g_volumeTable[pan + 0x40]`, else
/// `-g_volumeTable[0x40 - pan]`. Returns eax, or `None` where the original leaves eax.
pub unsafe fn set_buffer_pan(v: &Voice) -> Result<Option<i32>, Refused> {
    if AUDIO.sound_mode.get() & MODE_3D != 0 {
        return Ok(None);
    }
    if a3d() {
        return Err(BadInput("A3D (`nullsub_45`)").into());
    }
    // Converted: `rc2k_core::audio::levels::pan`.
    let value = rc2k_core::audio::levels::pan(v.pan, volumes)?;
    let b = buffer(v)?;
    Ok(Some(unsafe { com::call1(b, SET_PAN, value) }))
}

/// `SetFrequency` with the frequency shifted right by the format's halvings.
pub unsafe fn set_buffer_frequency(v: &Voice) -> Result<i32, Refused> {
    if a3d() {
        return Err(BadInput("A3D (`sub_40CEC5`)").into());
    }
    let b = buffer(v)?;
    // Converted: `rc2k_core::audio::levels::buffer_frequency`.
    let frequency = rc2k_core::audio::levels::buffer_frequency(v.frequency, v.format);
    Ok(unsafe { com::call1(b, SET_FREQUENCY, frequency) })
}

/// The 3D buffer's `SetPosition` with the place, deferred (1) while the voice sounds
/// (state 2), else at once (0).
pub unsafe fn set_buffer_place(v: &Voice) -> Result<i32, Refused> {
    if v.buffer_3d == 0 {
        return Err(BadInput("a voice without its 3D buffer").into());
    }
    // Converted: `rc2k_core::audio::buffers::place_deferral`.
    let deferred = rc2k_core::audio::buffers::place_deferral(v);
    unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(com::method(v.buffer_3d, SET_POSITION));
        Ok(f(v.buffer_3d, v.place[0], v.place[1], v.place[2], deferred))
    }
}
