//! `audio::describe_wave_format` (0x40A6A5): the `WAVEFORMATEX` for a sample's buffer.
//!
//! In:  eax = the format word, ebx = the frequency
//! Out: every register preserved
//!
//! `g_waveFormat` (0x506B30, 0x12 bytes) zeroed, then: tag 1 (PCM); samples a second the
//! frequency shifted right by the format's four low bits; channels from
//! `g_sampleChannelFlags` by the format's bits 8 .. 11 (none: fatal error 0x80A28541);
//! bits from `g_sampleBitsFlags` by bits 4 .. 7 (none: 0x80A28542); block align bits x
//! channels / 8 (the low sixteen bits stored); bytes a second that (as 32 bits) x the
//! samples a second. Five callers: `audio::make_buffer`, the stream's and the primary
//! buffer's set-ups.

use crate::audio::store::AudioState;
use crate::errors::{FatalError, Refused};

pub const WAVE_FORMAT_LEN: usize = 0x12;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WaveFormat {
    pub tag: u16,
    pub channels: u16,
    pub samples_per_second: u32,
    pub bytes_per_second: u32,
    pub block_align: u16,
    pub bits: u16,
    pub extra: u16,
}

const _: () = assert!(std::mem::size_of::<WaveFormat>() == WAVE_FORMAT_LEN);

/// The value a table of two {flag, value} entries has for the flag (`system::find_entry`).
pub fn wave_format(state: &AudioState, format: u32, frequency: u32) -> Result<WaveFormat, Refused> {
    use crate::audio::samples::flags;
    use rc2k_core::audio::samples::FormatError;
    // Converted: `rc2k_core::audio::samples::wave_format`.
    let bits = flags(&state.sample_bits_flags);
    let channels = flags(&state.sample_channel_flags);
    match rc2k_core::audio::samples::wave_format(format, frequency, &bits, &channels) {
        Ok(w) => Ok(WaveFormat {
            tag: 1,
            channels: w.channels,
            samples_per_second: w.samples_per_second,
            bytes_per_second: w.bytes_per_second,
            block_align: w.block_align,
            bits: w.bits,
            extra: 0,
        }),
        Err(e) => {
            let code = match e {
                FormatError::Channels => 0x80A2_8541,
                FormatError::Bits => 0x80A2_8542,
            };
            Err(FatalError { code, message: "" }.into())
        }
    }
}

pub fn describe_wave_format(
    state: &AudioState,
    format: u32,
    frequency: u32,
) -> Result<(), Refused> {
    let f = wave_format(state, format, frequency)?;
    state.wave_format.set(f);
    Ok(())
}

/// `g_waveFormat` as the platform's format (what `CreateSoundBuffer` and `SetFormat` read
/// of it).
pub fn platform_format(state: &AudioState) -> crate::platform_sound::Format {
    let w = state.wave_format.get();
    crate::platform_sound::Format {
        channels: w.channels as u32,
        bits: w.bits as u32,
        rate: w.samples_per_second,
    }
}
