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

use crate::audio::samples::{SAMPLE_BITS_FLAGS_RVA, SAMPLE_CHANNEL_FLAGS_RVA};
use crate::errors::{FatalError, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "audio::describe_wave_format",
    rva(0x40A6A5),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const WAVE_FORMAT_RVA: usize = rva(0x506B30); // g_waveFormat
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
    use rc2k_core::samples::FormatError;
    // Converted: `rc2k_core::samples::wave_format`.
    let bits = flags(SAMPLE_BITS_FLAGS_RVA);
    let channels = flags(SAMPLE_CHANNEL_FLAGS_RVA);
    match rc2k_core::samples::wave_format(format, frequency, &bits, &channels) {
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { describe_wave_format(r.eax, r.ebx) }?;
    Ok(Left::nothing())
}

fn region() -> [(u32, usize); 1] {
    [(game_addr(WAVE_FORMAT_RVA) as u32, WAVE_FORMAT_LEN)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &region(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = capture(&region());
    for bits in [0x10u32, 0x20] {
        for channels in [0x100u32, 0x200] {
            for shift in [0u32, 1, 2, 3] {
                for frequency in [22050u32, 44100, 100, 200_000] {
                    let format = bits | channels | shift;
                    let before = Regs {
                        eax: format | 0x1000,
                        ebx: frequency,
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    unsafe {
                        (game_addr(WAVE_FORMAT_RVA) as *mut u8).write_bytes(0xAA, WAVE_FORMAT_LEN);
                        call_original(&mut ro);
                    }
                    let theirs = capture(&region());
                    unsafe {
                        (game_addr(WAVE_FORMAT_RVA) as *mut u8).write_bytes(0xAA, WAVE_FORMAT_LEN)
                    };
                    let mine = unsafe { describe_wave_format(before.eax, frequency) };
                    let state = capture(&region());
                    t.check(
                        mine.is_ok() && theirs == state && ro.preserved_except(&before, 0),
                        || format!("format {format:#x}, frequency {frequency}: original {theirs:x?}; rust {mine:?} {state:x?}"),
                    );
                }
            }
        }
    }
    restore(&region(), &saved);
    t.finish()
}
