//! What `audio::set_buffer_volume`, `set_buffer_pan`, `set_buffer_frequency` and
//! `set_buffer_place` share: a voice's settings to its DirectSound buffer (one COM call each).

use crate::audio::own::AudioOwn;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

pub fn a3d(state: &AudioOwn) -> bool {
    state.use_a3d.get() == 1
}

/// The volume table's entry `n`, for the core.
pub fn volumes(state: &AudioOwn, n: u32) -> u32 {
    state.volume_table[n as usize].get()
}

fn buffer(v: &Voice) -> Result<u32, Refused> {
    if v.buffer == 0 {
        return Err(BadInput("a voice without its buffer").into());
    }
    Ok(v.buffer)
}

/// `SetVolume` with the volume table at `volume * volume_factor >> 6` (signed). Returns the result
/// code.
pub fn set_buffer_volume(state: &AudioOwn, v: &Voice) -> Result<i32, Refused> {
    if a3d(state) {
        return Err(BadInput("A3D (0x40CEFE)").into());
    }
    let level =
        rc2k_core::audio::levels::buffer_volume(v.volume, v.volume_factor, |n| volumes(state, n))?;
    let b = buffer(v)?;
    Ok(crate::platform_sound::set_volume(b, level as i32))
}

/// `SetPan`, nothing in 3D: a pan below 0 is the volume table's `pan + 0x40`, else minus its
/// `0x40 - pan`. Returns the result code, or `None` where the call is not made.
pub fn set_buffer_pan(state: &AudioOwn, v: &Voice) -> Result<Option<i32>, Refused> {
    if state.sound_mode.get() & MODE_3D != 0 {
        return Ok(None);
    }
    if a3d(state) {
        return Err(BadInput("A3D (`nullsub_45`)").into());
    }
    let value = rc2k_core::audio::levels::pan(v.pan, |n| volumes(state, n))?;
    let b = buffer(v)?;
    Ok(Some(crate::platform_sound::set_pan(b, value as i32)))
}

/// `SetFrequency` with the frequency shifted right by the format's halvings.
pub fn set_buffer_frequency(state: &AudioOwn, v: &Voice) -> Result<i32, Refused> {
    if a3d(state) {
        return Err(BadInput("A3D (0x40CEC5)").into());
    }
    let b = buffer(v)?;
    let frequency = rc2k_core::audio::levels::buffer_frequency(v.frequency, v.format);
    Ok(crate::platform_sound::set_frequency(b, frequency))
}

/// The 3D buffer's `SetPosition` with the place, deferred (1) while the voice sounds (state 2),
/// else at once (0).
pub fn set_buffer_place(v: &Voice) -> Result<i32, Refused> {
    if v.buffer_3d == 0 {
        return Err(BadInput("a voice without its 3D buffer").into());
    }
    // The platform takes it at once, deferred or not.
    let _deferred = rc2k_core::audio::buffers::place_deferral(v);
    Ok(crate::platform_sound::set_place(
        v.buffer_3d,
        v.place.map(f32::from_bits),
    ))
}
