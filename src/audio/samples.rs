//! Loading a sound's sample (command 3's logic, IDA 2026-10-14).
//!
//! `audio::read_sample` reads a file into the work buffer (`g_sampleBuffer`, 0x506930, of
//! `g_sampleBufferSize` bytes: the setting at 0x5068EC times 0x400, else 0x400) and leaves
//! what it found in `g_sampleFile` (`SampleFile`, 0x506938). A voice is then filled from
//! it, or from the voice it shares data with; the data is prepared in place for the sound
//! mode's quality and copied to the samples' memory (`g_sampleMemory`).
//!
//! Since phase 12 the bytes are Rust's (`audio::sample_data`): `SampleFile::data` and a
//! voice's `data` are ids there, not addresses.
//!
//! The format word: 0x10 eight bits, 0x20 sixteen; 0x100 mono, 0x200 stereo; the low four
//! bits how many times the data was halved (the frequency is shifted right by them).

use crate::audio::store::AUDIO;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::pointers::GamePtr;
use crate::session::{Dword, Shared};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SampleFile {
    /// The data's id (`audio::sample_data`).
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

/// Reads and parses a sample file into `buffer`: true for the carry clear (a sample was
/// found). The data's id goes in `out`.
pub fn read_sample(
    path: GamePtr,
    buffer: crate::audio::sample_data::Buffer,
    size: u32,
    out: &Shared<SampleFile>,
) -> Result<bool, Refused> {
    use crate::audio::sample_data as data;
    // SAFETY: `out` is one of the audio thread's records, borrowed only here.
    unsafe {
        let f = out.borrow();
        f.channels = 1;
        f.rate = 0x5622;
        f.bits = 16;
        let read = data::with_buffer(buffer, |b| {
            let into = b.get_mut(..size as usize).ok_or(crate::errors::BadInput(
                "a sample buffer smaller than its size",
            ))?;
            crate::files::read_file::read_file_into(path, into, 0)
        })?;
        if !read.ok {
            return Ok(false);
        }
        // Converted: `rc2k_core::audio::samples::parse`, on the work buffer's memory as the
        // original reads it (the data's place kept as its offset there).
        let mut core = rc2k_core::audio::samples::SampleFile {
            data: data::offset(f.data).unwrap_or(0),
            length: f.length,
            format: f.format,
            channels: f.channels,
            rate: f.rate,
            bits: f.bits,
            frame_bytes: f.frame_bytes,
            frames: f.frames,
        };
        let found = data::with_buffer(buffer, |file| {
            rc2k_core::audio::samples::parse(
                &&*file,
                read.bytes_read.unwrap_or(0),
                &mut core,
                &flags(&AUDIO.sample_bits_flags),
                &flags(&AUDIO.sample_channel_flags),
            )
        });
        let found = found?;
        *f = SampleFile {
            data: data::in_buffer(buffer, core.data),
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
pub fn flags(table: &[Dword; 4]) -> rc2k_core::audio::samples::Flags {
    [
        (table[0].get(), table[1].get()),
        (table[2].get(), table[3].get()),
    ]
}

/// The game's: the work buffer and `g_sampleFile`.
pub fn read_sample_in_game(path: GamePtr) -> Result<bool, Refused> {
    read_sample(
        path,
        crate::audio::sample_data::Buffer::Work,
        AUDIO.sample_buffer_size.get(),
        &AUDIO.sample_file,
    )
}

/// The voice of `count` from `voices` whose `data_key` is the record's (0: none).
pub fn find_shared_voice(record: &SampleRecord, count: u32) -> u32 {
    // Converted: `rc2k_core::audio::voices::find_shared`.
    rc2k_core::audio::voices::find_shared(
        &mut GamePool,
        count,
        crate::audio::voices::voice_id,
        record.data_key,
    )
    .unwrap_or(0)
}

/// `audio::fill_voice`: the voice `at` for the record (`record` is its address), from the
/// sample file or, with `shared`, from that voice.
pub fn fill_voice(
    at: u32,
    record_at: u32,
    shared: u32,
    file: &SampleFile,
    sound_mode: u32,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::voices::fill_voice`.
    let record: SampleRecord = GamePtr::new(record_at).read("a sample record")?;
    let from_file = rc2k_core::audio::voices::FileSample {
        data: file.data,
        length: file.length,
        format: file.format,
        frames: file.frames,
        rate: file.rate,
    };
    let failed = std::cell::Cell::new(None);
    let filled = rc2k_core::audio::voices::fill_voice(
        &mut GamePool,
        at,
        &record,
        shared,
        &from_file,
        sound_mode & 0x1_0000 != 0,
        // The record's level (`audio::store::VolumeLevel`).
        |factor| {
            crate::audio::store::volume_level(factor)
                .map(|level| level.get())
                .unwrap_or_else(|e| {
                    failed.set(failed.get().or(Some(e)));
                    0
                })
        },
    );
    if let Some(e) = failed.get() {
        return Err(e.into());
    }
    filled?;
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
pub fn prepare_sample(v: &mut Voice, mode: u32) -> Result<(), Refused> {
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
    // The data's `length` bytes worked on in place (the loops stay inside them).
    crate::audio::sample_data::with_data(v.data, |bytes| {
        let mut data = bytes
            .get_mut(..v.length as usize)
            .ok_or(crate::errors::BadInput("a sample longer than its memory"))?;
        rc2k_core::audio::samples::prepare(&mut shape, mode, &mut data)
            .map_err(crate::errors::BadInput::from)
    })??;
    v.length = shape.length;
    v.format = shape.format;
    v.frames = shape.frames;
    Ok(())
}
