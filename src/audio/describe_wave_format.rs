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
//! Compared in shadow mode. Self-test: the original on 18 formats and four frequencies.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::samples::{SAMPLE_BITS_FLAGS_RVA, SAMPLE_CHANNEL_FLAGS_RVA};
use crate::audio::store::Slot;
use crate::errors::{FatalError, Refused};
use crate::store::game_addr;

pub const WAVE_FORMAT_RVA: Slot = Slot::at(0x506B30); // g_waveFormat
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
pub fn wave_format(format: u32, frequency: u32) -> Result<WaveFormat, Refused> {
    use crate::audio::samples::flags;
    use rc2k_core::audio::samples::FormatError;
    // Converted: `rc2k_core::audio::samples::wave_format`.
    let bits = flags(SAMPLE_BITS_FLAGS_RVA);
    let channels = flags(SAMPLE_CHANNEL_FLAGS_RVA);
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

pub unsafe fn describe_wave_format(format: u32, frequency: u32) -> Result<(), Refused> {
    let f = wave_format(format, frequency)?;
    unsafe { (game_addr(WAVE_FORMAT_RVA) as *mut WaveFormat).write_unaligned(f) };
    Ok(())
}
