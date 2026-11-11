//! The game's mixer set up, and the two ways chosen by the mode:
//! `audio::set_up_primary_mixer` (0x40BAD2), `audio::open_way` (0x40AC3A),
//! `audio::close_way` (0x40AC58).

use crate::audio::owner::Audio;
use crate::errors::Refused;
use crate::system::owner::system;

pub const MODE_MIXER: u32 = 0x4_0000;
/// `SetCooperativeLevel`: write the primary buffer.
const WRITE_PRIMARY: u32 = 4;

/// `audio::set_up_primary_mixer`: nothing (true) outside the mixer's mode. Else
/// `release_primary`, cooperative level 4, `make_primary`, the mode's format
/// (`set_primary_format`), failing that eight bits, failing that mono too; the mode
/// kept so; the mixing rate (the format's), the most a turn mixes (rate x 2 / 50 x block
/// align, 40 ms), the shift (block / 2), the cursors; `GetCaps` (+0xC) of the primary into
/// `g_primaryCaps`, whose bytes are `g_primarySize`; `clear_channels`. A failure:
/// `close_primary_mixer` and false (the carry).
pub fn set_up_primary_mixer(audio: &Audio) -> Result<bool, Refused> {
    if audio.state.sound_mode.get() & MODE_MIXER == 0 {
        return Ok(true);
    }
    let failed = || crate::audio::restore::close_primary_mixer(audio).map(|_| false);
    crate::audio::sound_device::release_primary(&audio.state);
    let ds = audio.state.direct_sound.get();
    if crate::platform_sound::set_cooperative_level(
        ds,
        system().session.machine.window.get(),
        WRITE_PRIMARY,
    ) != 0
        || crate::audio::primary::make_primary(&audio.state).map(|made| !made)?
    {
        return failed();
    }
    // Converted: `rc2k_core::audio::device::{mixer_formats, mixer_figures}`.
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
    // The original reads the dword at +0xC (block align, bits); the figures use its low
    // word.
    let (most, shift) = mixer_figures(rate, wave.block_align as u32);
    audio.state.mix_most.set(most);
    audio.state.mix_shift.set(shift);
    audio.state.play_cursor.set(0);
    audio.state.write_cursor.set(most);
    let caps = &audio.state.primary_caps;
    caps.set_u32(0, 0x14);
    match crate::platform_sound::buffer_caps(audio.state.primary_buffer.get()) {
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

/// `audio::open_way`: the voices' own buffers: `clear_channels`; the mixer:
/// `set_up_primary_mixer`; neither: the carry. Returns the carry.
pub fn open_way(audio: &Audio, mode: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::audio::device::way`.
    use rc2k_core::audio::device::{Way, way};
    match way(mode) {
        // A jump into it: the carry is the one it leaves.
        Some(Way::OwnBuffers) => {
            crate::audio::clear_channels::clear_channels(audio);
            Ok(false)
        }
        Some(Way::Mixer) => Ok(!set_up_primary_mixer(audio)?),
        None => Ok(true),
    }
}

/// `audio::close_way`: as `open_way`, with `close_primary_mixer` for the mixer.
pub fn close_way(audio: &Audio, mode: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::audio::device::way`.
    use rc2k_core::audio::device::{Way, way};
    match way(mode) {
        // A jump into it: the carry is the one it leaves.
        Some(Way::OwnBuffers) => {
            crate::audio::clear_channels::clear_channels(audio);
            Ok(false)
        }
        Some(Way::Mixer) => crate::audio::restore::close_primary_mixer(audio),
        None => Ok(true),
    }
}
