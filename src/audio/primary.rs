//! The primary buffer made: `audio::make_primary` and its parts
//! `volume_index`, `set_primary_format`, `open_listener`. Logging and calls go through
//! `sound_device`'s named calls.

use crate::audio::store::AudioState;
use crate::errors::Refused;

const OPEN_TEXT: &std::ffi::CStr = c"AUD: open ds3d"; // "AUD: open ds3d"
const FAILED_TEXT: &std::ffi::CStr = c"AUD: open ds3d failed rc="; // "AUD: open ds3d failed rc="
const MODE_3D: u32 = crate::audio::voices::MODE_3D;

/// `audio::volume_index`: the first index of `g_volumeTable` whose volume is at least
/// `volume` (signed), or 0x40.
pub fn volume_index(state: &AudioState, volume: i32) -> u32 {
    // Converted: `rc2k_core::audio::samples::volume_index`.
    let table: [i32; 0x40] = std::array::from_fn(|n| state.volume_table[n].get() as i32);
    rc2k_core::audio::samples::volume_index(volume, &table)
}

/// `audio::set_primary_format`: `describe_wave_format` (the mode, `rate`), `SetFormat`
/// (+0x38). False for the carry (no primary buffer, or it failed).
pub fn set_primary_format(state: &AudioState, mode: u32, rate: u32) -> Result<bool, Refused> {
    {
        let p = state.primary_buffer.get();
        if p == 0 {
            return Ok(false);
        }
        crate::audio::describe_wave_format::describe_wave_format(state, mode, rate)?;
        let format = crate::audio::describe_wave_format::platform_format(state);
        Ok(crate::platform_sound::set_format(p, format) == 0)
    }
}

/// `audio::open_listener`: the 3D listener from the primary buffer (`QueryInterface`
/// with `g_iidListener3d` into `g_soundListener`) when `g_hasFpu` (`has_fpu`) is 1 and the
/// dword at 0x506B42 is not; else, or failing, logged and `release_sound_listener`. False
/// for the carry.
pub fn open_listener(state: &AudioState, has_fpu: u32) -> Result<bool, Refused> {
    crate::audio::sound_device::log_text(OPEN_TEXT, true);
    // Converted: `rc2k_core::audio::device::listener_refused`.
    let code = match rc2k_core::audio::device::listener_refused(has_fpu, || state.no_listener.get())
    {
        Some(code) => code,
        None => {
            let p = state.primary_buffer.get();
            match crate::platform_sound::query_interface(p, state.iid_listener.get_u32(0)) {
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

/// `audio::make_primary`: nothing with A3D; no DirectSound or a failed create: false (the
/// carry). `g_bufferDescription` zeroed, size 0x14, flags 0x81 (+0x10 in 3D);
/// `CreateSoundBuffer` into the primary; `GetVolume` (+0x18) into the kept volume;
/// `set_primary_format` (place 10, given the mode; its carry not looked at); a master volume
/// past 0x40 becomes `volume_index` of the kept one; it is shown and set
/// (`set_master_volume`, place 12); in 3D `open_listener` (place 13), whose failure
/// takes 3D out of the mode.
pub fn make_primary(
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    use crate::audio::make_buffer::BufferDescription;
    let state = &audio.state;
    if crate::audio::buffer_settings::a3d(state) {
        return Ok(true);
    }
    let ds = state.direct_sound.get();
    if ds == 0 {
        return Ok(false);
    }
    // Converted: `rc2k_core::audio::device` (`primary_flags`, `master_volume`,
    // `without_3d`).
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
        .set(crate::platform_sound::create_sound_buffer(
            description.flags,
            description.bytes,
            None,
        ));
    if let Ok(volume) = crate::platform_sound::volume(state.primary_buffer.get()) {
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
