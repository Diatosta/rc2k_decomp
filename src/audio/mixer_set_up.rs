//! The game's mixer set up, and the two ways chosen by the mode (IDA, 2026-10-15):
//! `audio::set_up_primary_mixer` (0x40BAD2), `audio::open_way` (0x40AC3A),
//! `audio::close_way` (0x40AC58).

use crate::audio::store::Slot;
use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::store::addr32 as a;

pub const PRIMARY_CAPS_RVA: Slot = Slot::at(0x506B08); // g_primaryCaps (DSBCAPS, 0x14 bytes)
pub const MODE_MIXER: u32 = 0x4_0000;
/// `SetCooperativeLevel`: write the primary buffer.
const WRITE_PRIMARY: u32 = 4;
const GET_CAPS: u32 = 0x0C;

/// Where the set-up's things are.
#[derive(Clone, Copy)]
pub struct MixerVars {
    pub mode: u32,
    pub direct_sound: u32,
    pub window: u32,
    pub primary: u32,
    /// `g_waveFormat` (samples a second at +4, block align at +0xC).
    pub wave: u32,
    pub rate: u32,
    pub most: u32,
    pub shift: u32,
    pub play: u32,
    pub write: u32,
    pub caps: u32,
    pub size: u32,
}

impl MixerVars {
    pub fn game() -> Self {
        use crate::audio::mixer::*;
        use crate::audio::voices::*;
        Self {
            mode: a(SOUND_MODE_RVA),
            direct_sound: a(crate::audio::make_buffer::DIRECT_SOUND_RVA),
            window: crate::session::MACHINE.window.addr() as u32,
            primary: a(PRIMARY_RVA),
            wave: a(crate::audio::describe_wave_format::WAVE_FORMAT_RVA),
            rate: a(MIX_RATE_RVA),
            most: a(MIX_MOST_RVA),
            shift: a(MIX_SHIFT_RVA),
            play: a(PLAY_CURSOR_RVA),
            write: a(WRITE_CURSOR_RVA),
            caps: a(PRIMARY_CAPS_RVA),
            size: a(PRIMARY_SIZE_RVA),
        }
    }
}

/// `audio::set_up_primary_mixer`: nothing (true) outside the mixer's mode. Else
/// `release_primary`, cooperative level 4, `make_primary`, the mode's format
/// (`set_primary_format`), failing that eight bits, failing that mono too; the mode
/// kept so; the mixing rate (the format's), the most a turn mixes (rate x 2 / 50 x block
/// align, 40 ms), the shift (block / 2), the cursors; `GetCaps` (+0xC) of the primary into
/// `g_primaryCaps`, whose bytes are `g_primarySize`; `clear_channels`. A failure:
/// `close_primary_mixer` and false (the carry).
pub unsafe fn set_up_primary_mixer(v: &MixerVars) -> Result<bool, Refused> {
    unsafe {
        if read(v.mode) & MODE_MIXER == 0 {
            return Ok(true);
        }
        let failed = || crate::audio::restore::close_primary_mixer_in_game().map(|_| false);
        crate::audio::restore::release_primary()?;
        let ds = read(v.direct_sound);
        let level: unsafe extern "system" fn(u32, u32, u32) -> i32 = std::mem::transmute(
            crate::audio::com::method(ds, crate::audio::sound_device::SET_COOPERATIVE_LEVEL),
        );
        if level(ds, read(v.window), WRITE_PRIMARY) != 0
            || crate::audio::restore::make_primary_failed()?
        {
            return failed();
        }
        // Converted: `rc2k_core::audio::device::{mixer_formats, mixer_figures}`.
        use rc2k_core::audio::device::{mixer_figures, mixer_formats};
        let mut chosen = None;
        for mode in mixer_formats(read(v.mode)) {
            if crate::audio::sound_device::set_primary_format(mode)? {
                chosen = Some(mode);
                break;
            }
        }
        let Some(mode) = chosen else {
            return failed();
        };
        write(v.mode, mode);
        let rate = read(v.wave + 4);
        write(v.rate, rate);
        let (most, shift) = mixer_figures(rate, read(v.wave + 0xC));
        write(v.most, most);
        write(v.shift, shift);
        write(v.play, 0);
        write(v.write, most);
        std::ptr::write_bytes(v.caps as *mut u8, 0, 0x14);
        write(v.caps, 0x14);
        if crate::audio::com::call1(read(v.primary), GET_CAPS, v.caps) != 0 {
            return failed();
        }
        write(v.size, read(v.caps + 8));
        crate::audio::restore::clear_channels();
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
            crate::audio::restore::clear_channels();
            Ok(false)
        }
        Some(Way::Mixer) => Ok(!unsafe { set_up_primary_mixer(&MixerVars::game()) }?),
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
            crate::audio::restore::clear_channels();
            Ok(false)
        }
        Some(Way::Mixer) => crate::audio::restore::close_primary_mixer_in_game(),
        None => Ok(true),
    }
}
