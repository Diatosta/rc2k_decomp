//! Loading a sound's sample (command 3's logic, IDA 2026-10-14).
//!
//! `audio::read_sample` reads a file into the work buffer (`g_sampleBuffer`, 0x506930, of
//! `g_sampleBufferSize` bytes: the setting at 0x5068EC times 0x400, else 0x400) and leaves
//! what it found in `g_sampleFile` (`SampleFile`, 0x506938). A voice is then filled from
//! it, or from the voice it shares data with; the data is prepared in place for the sound
//! mode's quality and copied to the samples' memory (`g_sampleMemory`).
//!
//! The format word: 0x10 eight bits, 0x20 sixteen; 0x100 mono, 0x200 stereo; the low four
//! bits how many times the data was halved (the frequency is shifted right by them).

use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{game_addr, game_read, rva};

pub const SAMPLE_FILE_RVA: usize = rva(0x506938); // g_sampleFile
pub const BUFFER_RVA: usize = rva(0x506930); // g_sampleBuffer
pub const BUFFER_SIZE_RVA: usize = rva(0x506934); // g_sampleBufferSize
pub const SAMPLE_BITS_FLAGS_RVA: usize = rva(0x4F82E0); // g_sampleBitsFlags: {flag, bits} x 2
pub const SAMPLE_CHANNEL_FLAGS_RVA: usize = rva(0x4F82F0); // g_sampleChannelFlags: {flag, channels} x 2

pub const EIGHT_BITS: u32 = 0x10;
pub const MONO: u32 = 0x100;
pub const SHIFT: u32 = 0xF;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SampleFile {
    pub data: u32,
    pub length: u32,
    pub format: u32,
    pub channels: u32,
    pub rate: u32,
    pub bits: u32,
    pub frame_bytes: u32,
    pub frames: u32,
}

const _: () = assert!(std::mem::size_of::<SampleFile>() == 0x20);

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// Reads and parses a sample file: true for the carry clear (a sample was found).
pub unsafe fn read_sample(
    path: *const u8,
    buffer: u32,
    size: u32,
    out: *mut SampleFile,
) -> Result<bool, Refused> {
    unsafe {
        let f = &mut *out;
        f.channels = 1;
        f.rate = 0x5622;
        f.bits = 16;
        let read = crate::files::read_file::read_file(path, buffer as *mut u8, size, 0)?;
        if !read.ok {
            return Ok(false);
        }
        // Converted: `rc2k_core::audio::samples::parse`, on the work buffer's memory as the
        // original reads it.
        let mut core = rc2k_core::audio::samples::SampleFile {
            data: f.data.wrapping_sub(buffer),
            length: f.length,
            format: f.format,
            channels: f.channels,
            rate: f.rate,
            bits: f.bits,
            frame_bytes: f.frame_bytes,
            frames: f.frames,
        };
        let file = crate::bridge::Raw::new(buffer as *const u8);
        let found = rc2k_core::audio::samples::parse(
            &file,
            read.bytes_read.unwrap_or(0),
            &mut core,
            &flags(SAMPLE_BITS_FLAGS_RVA),
            &flags(SAMPLE_CHANNEL_FLAGS_RVA),
        );
        let found = crate::bridge::read_through(found);
        *f = SampleFile {
            data: core.data.wrapping_add(buffer),
            length: core.length,
            format: core.format,
            channels: core.channels,
            rate: core.rate,
            bits: core.bits,
            frame_bytes: core.frame_bytes,
            frames: core.frames,
        };
        Ok(found)
    }
}

/// A table of two {flag, value} entries, as the core takes it.
pub fn flags(table: usize) -> rc2k_core::audio::samples::Flags {
    let t: [u32; 4] = unsafe { game_read(table) };
    [(t[0], t[1]), (t[2], t[3])]
}

/// The game's: the work buffer and `g_sampleFile`.
pub unsafe fn read_sample_in_game(path: *const u8) -> Result<bool, Refused> {
    unsafe {
        read_sample(
            path,
            game_read(BUFFER_RVA),
            game_read(BUFFER_SIZE_RVA),
            game_addr(SAMPLE_FILE_RVA) as *mut SampleFile,
        )
    }
}

/// The voice of `count` from `voices` whose `data_key` is the record's (0: none).
pub unsafe fn find_shared_voice(record: &SampleRecord, voices: u32, count: u32) -> u32 {
    // Converted: `rc2k_core::audio::voices::find_shared`.
    let mut pool = unsafe { crate::audio::voices::GamePool::new(&World::game()) };
    let size = std::mem::size_of::<Voice>() as u32;
    rc2k_core::audio::voices::find_shared(&mut pool, count, |n| voices + n * size, record.data_key)
        .unwrap_or(0)
}

/// `audio::fill_voice`: the voice `at` for the record (`record` is its address), from the
/// sample file or, with `shared`, from that voice.
pub unsafe fn fill_voice(
    at: u32,
    record_at: u32,
    shared: u32,
    file: &SampleFile,
    sound_mode: u32,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::voices::fill_voice`.
    let record = unsafe { *(record_at as *const SampleRecord) };
    let from_file = rc2k_core::audio::voices::FileSample {
        data: file.data,
        length: file.length,
        format: file.format,
        frames: file.frames,
        rate: file.rate,
    };
    let mut pool = unsafe { crate::audio::voices::GamePool::new(&World::game()) };
    rc2k_core::audio::voices::fill_voice(
        &mut pool,
        at,
        &record,
        shared,
        &from_file,
        sound_mode & 0x1_0000 != 0,
        |factor| unsafe { dword(factor) },
    )?;
    Ok(())
}

/// `audio::sample_mode`: the mode a voice's sample is prepared for. Converted:
/// `rc2k_core::audio::samples::sample_mode`.
pub fn sample_mode(mode: u32, record: Option<&SampleRecord>) -> u32 {
    rc2k_core::audio::samples::sample_mode(mode, record.map(|r| (r.number, r.group)))
}

/// `audio::prepare_sample`: the voice's data, in place, scaled by `scale` / 64, halved
/// until its shift is the mode's, and made eight bits if the mode asks. Converted:
/// `rc2k_core::audio::samples::prepare`, on the voice's data in game memory.
pub unsafe fn prepare_sample(v: &mut Voice, mode: u32) -> Result<(), Refused> {
    if v.data == 0 || v.length == 0 || v.data_key == 0 {
        return Ok(());
    }
    let mut shape = rc2k_core::audio::samples::Shape {
        length: v.length,
        format: v.format,
        frames: v.frames,
        scale: v.scale,
        own_frequency: v.own_frequency,
    };
    let mut data = unsafe { crate::bridge::RawMut::new(v.data as *mut u8) };
    rc2k_core::audio::samples::prepare(&mut shape, mode, &mut data)?;
    v.length = shape.length;
    v.format = shape.format;
    v.frames = shape.frames;
    Ok(())
}
