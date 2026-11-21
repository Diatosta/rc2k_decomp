//! The game's mixer set up, and the two ways chosen by the mode: `audio::set_up_primary_mixer`,
//! `audio::open_way`, `audio::close_way`, `audio::close_primary_mixer`.

use crate::audio::own::AudioThread;
use crate::errors::Refused;
use crate::system::owner::System;

pub(super) const MODE_MIXER: u32 = 0x4_0000;
/// `SetCooperativeLevel`: write the primary buffer.
const WRITE_PRIMARY: u32 = 4;

/// Nothing (true) outside the mixer's mode. Else the primary buffer is released and made again with
/// the mode's format (failing that eight bits, failing that mono too), and the mixing rate, the
/// most a turn mixes (40 ms), the shift and the cursors are set; the channels are cleared. A
/// failure closes the mixer and returns false.
pub fn set_up_primary_mixer(audio: &AudioThread, system: &System) -> Result<bool, Refused> {
    if audio.state.sound_mode.get() & MODE_MIXER == 0 {
        return Ok(true);
    }
    let failed = || close_primary_mixer(audio, system).map(|_| false);
    crate::audio::sound_device::release_primary(&audio.state);
    let ds = audio.state.direct_sound.get();
    if crate::platform::sound::set_cooperative_level(
        ds,
        system.session.host.window.get(),
        WRITE_PRIMARY,
    ) != 0
        || crate::audio::primary::make_primary(audio, system).map(|made| !made)?
    {
        return failed();
    }
    use rc2k_core::audio::device::{mixer_figures, mixer_formats};
    let mut chosen = None;
    for mode in mixer_formats(audio.state.sound_mode.get()) {
        if crate::audio::sound_device::set_primary_format(&audio.state, mode)? {
            chosen = Some(mode);
            break;
        }
    }
    let Some(mode) = chosen else {
        return failed();
    };
    audio.state.sound_mode.set(mode);
    let wave = audio.state.wave_format.get();
    let rate = wave.samples_per_second;
    audio.state.mix_rate.set(rate);
    let (most, shift) = mixer_figures(rate, wave.block_align as u32);
    audio.state.mix_most.set(most);
    audio.state.mix_shift.set(shift);
    audio.state.play_cursor.set(0);
    audio.state.write_cursor.set(most);
    let caps = &audio.state.primary_caps;
    caps.set_u32(0, 0x14);
    match crate::platform::sound::buffer_caps(audio.state.primary_buffer.get()) {
        Ok((flags, bytes)) => {
            for (n, value) in [(4, flags), (8, bytes), (0xC, 0), (0x10, 0)] {
                caps.set_u32(n, value);
            }
        }
        Err(_) => {
            for n in [4, 8, 0xC, 0x10] {
                caps.set_u32(n, 0);
            }
            return failed();
        }
    }
    audio.state.primary_size.set(caps.get_u32(8));
    crate::audio::clear_channels::clear_channels(audio);
    Ok(true)
}

/// The voices' own buffers: `clear_channels`; the mixer: `set_up_primary_mixer`; neither: failure.
pub fn open_way(audio: &AudioThread, system: &System, mode: u32) -> Result<bool, Refused> {
    use rc2k_core::audio::device::{Way, way};
    match way(mode) {
        // The result is the one the callee leaves.
        Some(Way::OwnBuffers) => {
            crate::audio::clear_channels::clear_channels(audio);
            Ok(false)
        }
        Some(Way::Mixer) => Ok(!set_up_primary_mixer(audio, system)?),
        None => Ok(true),
    }
}

/// As `open_way`, with `close_primary_mixer` for the mixer.
pub fn close_way(audio: &AudioThread, system: &System, mode: u32) -> Result<bool, Refused> {
    use rc2k_core::audio::device::{Way, way};
    match way(mode) {
        // The result is the one the callee leaves.
        Some(Way::OwnBuffers) => {
            crate::audio::clear_channels::clear_channels(audio);
            Ok(false)
        }
        Some(Way::Mixer) => close_primary_mixer(audio, system),
        None => Ok(true),
    }
}

/// The channels cleared; in the mixer's mode with a DirectSound: `release_primary`, cooperative
/// level 3, `make_primary`. Returns failure of `make_primary`, when that ran.
pub fn close_primary_mixer(audio: &AudioThread, system: &System) -> Result<bool, Refused> {
    let state = &audio.state;
    crate::audio::clear_channels::clear_channels(audio);
    if !rc2k_core::audio::buffers::remakes_primary(state.sound_mode.get(), || {
        state.direct_sound.get()
    }) {
        return Ok(false);
    }
    crate::audio::sound_device::release_primary(state);
    let ds = state.direct_sound.get();
    crate::platform::sound::set_cooperative_level(ds, system.session.host.window.get(), 3);
    Ok(!crate::audio::primary::make_primary(audio, system)?)
}
