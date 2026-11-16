//! The primary buffer made: `audio::make_primary` and its parts `volume_index`,
//! `set_primary_format`, `open_listener`.

use crate::audio::own::AudioOwn;
use crate::errors::Refused;

const OPEN_TEXT: &std::ffi::CStr = c"AUD: open ds3d";
const FAILED_TEXT: &std::ffi::CStr = c"AUD: open ds3d failed rc=";
const MODE_3D: u32 = crate::audio::voices::MODE_3D;

/// The first index of the volume table whose volume is at least `volume` (signed), or 0x40.
pub fn volume_index(state: &AudioOwn, volume: i32) -> u32 {
    let table: [i32; 0x40] = std::array::from_fn(|n| state.volume_table[n].get() as i32);
    rc2k_core::audio::samples::volume_index(volume, &table)
}

/// `describe_wave_format` for the mode and `rate`, then `SetFormat`. False when there is no
/// primary buffer or it failed.
pub fn set_primary_format(state: &AudioOwn, mode: u32, rate: u32) -> Result<bool, Refused> {
    {
        let p = state.primary_buffer.get();
        if p == 0 {
            return Ok(false);
        }
        crate::audio::describe_wave_format::describe_wave_format(state, mode, rate)?;
        let format = crate::audio::describe_wave_format::platform_format(state);
        Ok(crate::platform::sound::set_format(p, format) == 0)
    }
}

/// The 3D listener from the primary buffer when the machine has an FPU and the listener setting is
/// not off; else, or failing, logged and released. False when it failed.
pub fn open_listener(state: &AudioOwn, has_fpu: u32) -> Result<bool, Refused> {
    crate::audio::sound_device::log_text(OPEN_TEXT, true);
    let code = match rc2k_core::audio::device::listener_refused(has_fpu, || state.no_listener.get())
    {
        Some(code) => code,
        None => {
            let p = state.primary_buffer.get();
            match crate::platform::sound::query_interface(p, state.iid_listener.get_u32(0)) {
                Ok(listener) => {
                    state.sound_listener.set(listener);
                    return Ok(true);
                }
                Err(r) => {
                    state.sound_listener.set(0);
                    r as u32
                }
            }
        }
    };
    crate::audio::sound_device::log_text(FAILED_TEXT, false);
    crate::audio::sound_device::log_hex(code, true);
    crate::audio::sound_device::release_listener(state);
    Ok(false)
}

/// No DirectSound or a failed create: false. The buffer description is zeroed,
/// size 0x14, flags 0x81 (+0x10 in 3D); the primary buffer is created, its volume kept; the format
/// set; a master volume past 0x40 becomes `volume_index` of the kept one; it is shown and set; in
/// 3D the listener is opened, whose failure takes 3D out of the mode.
pub fn make_primary(
    audio: &crate::audio::own::AudioThread,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    use crate::audio::make_buffer::BufferDescription;
    let state = &audio.state;
    let ds = state.direct_sound.get();
    if ds == 0 {
        return Ok(false);
    }
    use rc2k_core::audio::device::{master_volume, primary_flags, without_3d};
    state.buffer_description.set(BufferDescription {
        size: 0x14,
        flags: primary_flags(state.sound_mode.get()),
        ..BufferDescription::default()
    });
    let description = state.buffer_description.get();
    // (Its format is 0: the device's own.)
    state
        .primary_buffer
        .set(crate::platform::sound::create_sound_buffer(
            description.flags,
            description.bytes,
            None,
        ));
    if let Ok(volume) = crate::platform::sound::volume(state.primary_buffer.get()) {
        state.primary_volume_kept.set(volume as u32);
    }
    crate::audio::sound_device::set_primary_format(state, state.sound_mode.get())?;
    let (volume, stored) = master_volume(state.master_volume.get(), || {
        volume_index(state, state.primary_volume_kept.get() as i32)
    });
    if stored {
        state.master_volume.set(volume);
    }
    audio.levels.master_volume_shown.set(volume);
    crate::audio::set_master_volume::set_master_volume(state, volume);
    if state.sound_mode.get() & MODE_3D != 0
        && !crate::audio::sound_device::open_listener(state, system)?
    {
        state.sound_mode.set(without_3d(state.sound_mode.get()));
    }
    Ok(true)
}
