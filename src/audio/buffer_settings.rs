//! What `audio::set_buffer_volume`, `set_buffer_pan`, `set_buffer_frequency` and
//! `set_buffer_place` share: a voice's settings to its DirectSound buffer (one COM call
//! each).

use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

pub fn a3d() -> bool {
    crate::audio::owner::audio().state.use_a3d.get() == 1
}

/// `g_volumeTable`'s entry `n`, for the core.
pub fn volumes(n: u32) -> u32 {
    crate::audio::owner::audio().state.volume_table[n as usize].get()
}

fn buffer(v: &Voice) -> Result<u32, Refused> {
    if v.buffer == 0 {
        return Err(BadInput("a voice without its buffer").into());
    }
    Ok(v.buffer)
}

/// `SetVolume` with `g_volumeTable[volume * volume_factor >> 6]` (signed). Returns eax.
pub fn set_buffer_volume(v: &Voice) -> Result<i32, Refused> {
    if a3d() {
        return Err(BadInput("A3D (`sub_40CEFE`)").into());
    }
    // Converted: `rc2k_core::audio::levels::buffer_volume`.
    let level = rc2k_core::audio::levels::buffer_volume(v.volume, v.volume_factor, volumes)?;
    let b = buffer(v)?;
    Ok(crate::platform_sound::set_volume(b, level as i32))
}

/// `SetPan`, nothing in 3D: a pan below 0 is `g_volumeTable[pan + 0x40]`, else
/// `-g_volumeTable[0x40 - pan]`. Returns eax, or `None` where the original leaves eax.
pub fn set_buffer_pan(v: &Voice) -> Result<Option<i32>, Refused> {
    if crate::audio::owner::audio().state.sound_mode.get() & MODE_3D != 0 {
        return Ok(None);
    }
    if a3d() {
        return Err(BadInput("A3D (`nullsub_45`)").into());
    }
    // Converted: `rc2k_core::audio::levels::pan`.
    let value = rc2k_core::audio::levels::pan(v.pan, volumes)?;
    let b = buffer(v)?;
    Ok(Some(crate::platform_sound::set_pan(b, value as i32)))
}

/// `SetFrequency` with the frequency shifted right by the format's halvings.
pub fn set_buffer_frequency(v: &Voice) -> Result<i32, Refused> {
    if a3d() {
        return Err(BadInput("A3D (`sub_40CEC5`)").into());
    }
    let b = buffer(v)?;
    // Converted: `rc2k_core::audio::levels::buffer_frequency`.
    let frequency = rc2k_core::audio::levels::buffer_frequency(v.frequency, v.format);
    Ok(crate::platform_sound::set_frequency(b, frequency))
}

/// The 3D buffer's `SetPosition` with the place, deferred (1) while the voice sounds
/// (state 2), else at once (0).
pub fn set_buffer_place(v: &Voice) -> Result<i32, Refused> {
    if v.buffer_3d == 0 {
        return Err(BadInput("a voice without its 3D buffer").into());
    }
    // Converted: `rc2k_core::audio::buffers::place_deferral`.
    // (Deferred or not: the platform takes it at once.)
    let _deferred = rc2k_core::audio::buffers::place_deferral(v);
    Ok(crate::platform_sound::set_place(
        v.buffer_3d,
        v.place.map(f32::from_bits),
    ))
}
