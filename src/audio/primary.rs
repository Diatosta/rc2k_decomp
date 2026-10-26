//! The primary buffer made (IDA, 2026-10-15): `audio::make_primary` and its parts
//! `volume_index`, `set_primary_format`, `open_listener`. Logging and calls go through
//! `sound_device`'s named calls.

use crate::audio::store::AUDIO;
use crate::errors::Refused;
use crate::menu::option_work_store::OPTION_WORK;

const OPEN_TEXT: &std::ffi::CStr = c"AUD: open ds3d"; // "AUD: open ds3d"
const FAILED_TEXT: &std::ffi::CStr = c"AUD: open ds3d failed rc="; // "AUD: open ds3d failed rc="
const MODE_3D: u32 = crate::audio::voices::MODE_3D;

/// `audio::volume_index`: the first index of `g_volumeTable` whose volume is at least
/// `volume` (signed), or 0x40.
pub fn volume_index(volume: i32) -> u32 {
    // Converted: `rc2k_core::audio::samples::volume_index`.
    let table: [i32; 0x40] = std::array::from_fn(|n| AUDIO.volume_table[n].get() as i32);
    rc2k_core::audio::samples::volume_index(volume, &table)
}

/// `audio::set_primary_format`: `describe_wave_format` (the mode, `rate`), `SetFormat`
/// (+0x38). False for the carry (no primary buffer, or it failed).
pub fn set_primary_format(mode: u32, rate: u32) -> Result<bool, Refused> {
    {
        let p = AUDIO.primary_buffer.get();
        if p == 0 {
            return Ok(false);
        }
        crate::audio::describe_wave_format::describe_wave_format(mode, rate)?;
        let format = crate::audio::describe_wave_format::platform_format();
        Ok(crate::platform_sound::set_format(p, format) == 0)
    }
}

/// `audio::open_listener`: the 3D listener from the primary buffer (`QueryInterface`
/// with `g_iidListener3d` into `g_soundListener`) when `g_hasFpu` (`has_fpu`) is 1 and the
/// dword at 0x506B42 is not; else, or failing, logged and `release_sound_listener`. False
/// for the carry.
pub fn open_listener(has_fpu: u32) -> Result<bool, Refused> {
    crate::audio::sound_device::log_text(crate::pointers::game_address(OPEN_TEXT.as_ptr()), true);
    // Converted: `rc2k_core::audio::device::listener_refused`.
    let code = match rc2k_core::audio::device::listener_refused(has_fpu, || AUDIO.no_listener.get())
    {
        Some(code) => code,
        None => {
            let p = AUDIO.primary_buffer.get();
            match crate::platform_sound::query_interface(p, AUDIO.iid_listener.get_u32(0)) {
                Ok(listener) => {
                    AUDIO.sound_listener.set(listener);
                    return Ok(true);
                }
                Err(r) => {
                    AUDIO.sound_listener.set(0);
                    r as u32
                }
            }
        }
    };
    crate::audio::sound_device::log_text(
        crate::pointers::game_address(FAILED_TEXT.as_ptr()),
        false,
    );
    crate::audio::sound_device::log_hex(code, true);
    crate::audio::sound_device::release_listener();
    Ok(false)
}

/// `audio::make_primary`: nothing with A3D; no DirectSound or a failed create: false (the
/// carry). `g_bufferDescription` zeroed, size 0x14, flags 0x81 (+0x10 in 3D);
/// `CreateSoundBuffer` into the primary; `GetVolume` (+0x18) into the kept volume;
/// `set_primary_format` (place 10, eax the mode; its carry not looked at); a master volume
/// past 0x40 becomes `volume_index` of the kept one; it is shown and set
/// (`set_master_volume`, place 12, eax); in 3D `open_listener` (place 13), whose failure
/// takes 3D out of the mode.
pub fn make_primary() -> Result<bool, Refused> {
    use crate::audio::make_buffer::BufferDescription;
    if crate::audio::buffer_settings::a3d() {
        return Ok(true);
    }
    let ds = AUDIO.direct_sound.get();
    if ds == 0 {
        return Ok(false);
    }
    // Converted: `rc2k_core::audio::device` (`primary_flags`, `master_volume`,
    // `without_3d`).
    use rc2k_core::audio::device::{master_volume, primary_flags, without_3d};
    AUDIO.buffer_description.set(BufferDescription {
        size: 0x14,
        flags: primary_flags(AUDIO.sound_mode.get()),
        ..BufferDescription::default()
    });
    let description = AUDIO.buffer_description.get();
    // (Its format is 0: the device's own.)
    AUDIO
        .primary_buffer
        .set(crate::platform_sound::create_sound_buffer(
            description.flags,
            description.bytes,
            None,
        ));
    if let Ok(volume) = crate::platform_sound::volume(AUDIO.primary_buffer.get()) {
        AUDIO.primary_volume_kept.set(volume as u32);
    }
    crate::audio::sound_device::set_primary_format(AUDIO.sound_mode.get())?;
    let (volume, stored) = master_volume(AUDIO.master_volume.get(), || {
        volume_index(AUDIO.primary_volume_kept.get() as i32)
    });
    if stored {
        AUDIO.master_volume.set(volume);
    }
    OPTION_WORK.master_volume_shown.set(volume);
    crate::audio::set_master_volume::set_master_volume_in_game(volume);
    if AUDIO.sound_mode.get() & MODE_3D != 0 && !crate::audio::sound_device::open_listener()? {
        AUDIO.sound_mode.set(without_3d(AUDIO.sound_mode.get()));
    }
    Ok(true)
}
