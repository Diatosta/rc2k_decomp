//! The game's mixer set up, and the two ways chosen by the mode (IDA, 2026-10-15):
//! `audio::set_up_primary_mixer` (0x40BAD2), `audio::open_way` (0x40AC3A),
//! `audio::close_way` (0x40AC58).

use crate::audio::store::AUDIO;
use crate::errors::Refused;
use crate::session::MACHINE;

pub const MODE_MIXER: u32 = 0x4_0000;
/// `SetCooperativeLevel`: write the primary buffer.
const WRITE_PRIMARY: u32 = 4;
const GET_CAPS: u32 = 0x0C;

/// `audio::set_up_primary_mixer`: nothing (true) outside the mixer's mode. Else
/// `release_primary`, cooperative level 4, `make_primary`, the mode's format
/// (`set_primary_format`), failing that eight bits, failing that mono too; the mode
/// kept so; the mixing rate (the format's), the most a turn mixes (rate x 2 / 50 x block
/// align, 40 ms), the shift (block / 2), the cursors; `GetCaps` (+0xC) of the primary into
/// `g_primaryCaps`, whose bytes are `g_primarySize`; `clear_channels`. A failure:
/// `close_primary_mixer` and false (the carry).
pub fn set_up_primary_mixer() -> Result<bool, Refused> {
    unsafe {
        if AUDIO.sound_mode.get() & MODE_MIXER == 0 {
            return Ok(true);
        }
        let failed = || crate::audio::restore::close_primary_mixer_in_game().map(|_| false);
        crate::audio::sound_device::release_primary()?;
        let ds = AUDIO.direct_sound.get();
        let level: unsafe extern "system" fn(u32, u32, u32) -> i32 = std::mem::transmute(
            crate::audio::com::method(ds, crate::audio::sound_device::SET_COOPERATIVE_LEVEL),
        );
        if level(ds, MACHINE.window.get(), WRITE_PRIMARY) != 0
            || crate::audio::primary::make_primary().map(|made| !made)?
        {
            return failed();
        }
        // Converted: `rc2k_core::audio::device::{mixer_formats, mixer_figures}`.
        use rc2k_core::audio::device::{mixer_figures, mixer_formats};
        let mut chosen = None;
        for mode in mixer_formats(AUDIO.sound_mode.get()) {
            if crate::audio::sound_device::set_primary_format(mode)? {
                chosen = Some(mode);
                break;
            }
        }
        let Some(mode) = chosen else {
            return failed();
        };
        AUDIO.sound_mode.set(mode);
        let wave = AUDIO.wave_format.get();
        let rate = wave.samples_per_second;
        AUDIO.mix_rate.set(rate);
        // The original reads the dword at +0xC (block align, bits); the figures use its low
        // word.
        let (most, shift) = mixer_figures(rate, wave.block_align as u32);
        AUDIO.mix_most.set(most);
        AUDIO.mix_shift.set(shift);
        AUDIO.play_cursor.set(0);
        AUDIO.write_cursor.set(most);
        let caps = &AUDIO.primary_caps;
        std::ptr::write_bytes(caps.as_ptr(), 0, 0x14);
        caps.set_u32(0, 0x14);
        if crate::audio::com::call1(AUDIO.primary_buffer.get(), GET_CAPS, caps.addr()) != 0 {
            return failed();
        }
        AUDIO.primary_size.set(caps.get_u32(8));
        crate::audio::clear_channels::clear_channels();
        Ok(true)
    }
}

/// `audio::open_way`: the voices' own buffers: `clear_channels`; the mixer:
/// `set_up_primary_mixer`; neither: the carry. Returns the carry.
pub fn open_way(mode: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::audio::device::way`.
    use rc2k_core::audio::device::{Way, way};
    match way(mode) {
        // A jump into it: the carry is the one it leaves.
        Some(Way::OwnBuffers) => {
            crate::audio::clear_channels::clear_channels();
            Ok(false)
        }
        Some(Way::Mixer) => Ok(!set_up_primary_mixer()?),
        None => Ok(true),
    }
}

/// `audio::close_way`: as `open_way`, with `close_primary_mixer` for the mixer.
pub fn close_way(mode: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::audio::device::way`.
    use rc2k_core::audio::device::{Way, way};
    match way(mode) {
        // A jump into it: the carry is the one it leaves.
        Some(Way::OwnBuffers) => {
            crate::audio::clear_channels::clear_channels();
            Ok(false)
        }
        Some(Way::Mixer) => crate::audio::restore::close_primary_mixer_in_game(),
        None => Ok(true),
    }
}
