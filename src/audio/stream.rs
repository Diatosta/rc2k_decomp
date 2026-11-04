//! The music stream (IDA, 2026-10-14): a WAV file played through a looping DirectSound
//! buffer that is refilled from the file each turn. Two players share the code, with the
//! player in esi: `g_soundStream` (0x506960, command 9, `audio::stream_command`) and
//! `g_musicStream` (0x506990, command 0xB, `audio::music_command`).
//!
//! `audio::start_stream` reads the file's head into `g_streamHeader` (0xC00 bytes, taken
//! once), opens the file at its data, makes a buffer of `bytes` and plays it looping, with
//! the write place `step` ahead of the play cursor. `audio::feed_stream` writes into the
//! buffer what the play cursor has moved over since the last turn (at most `step`), read
//! from the file; after the file's end, silence, `tail` bytes of it, then the player
//! stops. Each function begins `cmp g_useA3d, 1; jz` an A3D twin: refused.

use crate::audio::buffer_settings::a3d;
use crate::audio::store::AUDIO;
use crate::errors::{BadInput, Refused};

pub const HEADER_LEN: u32 = 0xC00;

pub use rc2k_core::audio::streams::AudioStream;

pub fn not_a3d(twin: &'static str) -> Result<(), Refused> {
    if a3d() {
        return Err(BadInput(twin).into());
    }
    Ok(())
}

pub fn volume_of(index: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::audio::levels::stream_volume`.
    Ok(rc2k_core::audio::levels::stream_volume(
        index,
        crate::audio::buffer_settings::volumes,
    )?)
}

/// What `fill_stream_part` did: `None`, nothing (no place or no bytes); else the bytes read
/// and whether the rest was filled with silence.
pub type Filled = Option<(u32, bool)>;

/// `audio::fill_stream_part`: `into`'s bytes (a locked part's) from the file; what is not
/// there becomes silence (0, or 0x80 for eight bits), and a read that found nothing uses
/// up `tail` (the player stops when it goes below 0). An empty `into` is no part.
pub fn fill_stream_part(s: &mut AudioStream, into: &mut [u8]) -> Filled {
    if into.is_empty() {
        return None;
    }
    // Converted: `rc2k_core::audio::streams::AudioStream::after_read`.
    let (read, _) = crate::files::handle::read_slice(s.file, into);
    let Some(quiet) = s.after_read(into.len() as u32, read) else {
        return Some((read, false));
    };
    into[read as usize..].fill(quiet);
    Some((read, true))
}

/// The bytes of a part `fill_stream_part` wrote.
fn written(filled: Filled, bytes: usize) -> usize {
    match filled {
        None => 0,
        Some((read, false)) => read as usize,
        Some((_, true)) => bytes,
    }
}

/// `audio::stop_stream`: not playing; the buffer stopped and released; the file closed.
pub fn stop_stream(s: &mut AudioStream) -> Result<(), Refused> {
    not_a3d("A3D (0x40D1D0)")?;
    // Converted: `rc2k_core::audio::streams::AudioStream::stop`.
    let (buffer, file) = s.stop();
    if let Some(b) = buffer {
        crate::platform_sound::stop(b);
        crate::platform_sound::release(b);
    }
    if let Some(f) = file {
        crate::files::handle::close(f);
    }
    Ok(())
}

/// `audio::set_stream_volume`: `SetVolume` with the player's volume.
pub fn set_stream_volume(s: &AudioStream) -> Result<(), Refused> {
    not_a3d("A3D (0x40D247)")?;
    if s.buffer == 0 {
        return Err(BadInput("a stream without its buffer").into());
    }
    let v = volume_of(s.volume)?;
    crate::platform_sound::set_volume(s.buffer, v as i32);
    Ok(())
}

/// `audio::start_stream`: the file at `path` opened and played. False for the carry (the
/// player stopped).
pub fn start_stream(s: &mut AudioStream, path: crate::pointers::GamePtr) -> Result<bool, Refused> {
    use crate::audio::describe_wave_format::describe_wave_format;
    use crate::audio::make_buffer::BufferDescription;
    not_a3d("A3D (0x40CF6B)")?;
    // Checked before anything is done: the volume it sets.
    let volume = volume_of(s.volume)?;
    let failed = |s: &mut AudioStream| stop_stream(s).map(|_| false);
    let ds = AUDIO.direct_sound.get();
    if ds == 0 {
        return failed(s);
    }
    // The header's buffer: Rust's (`audio::sample_data`), taken once and counted as the
    // game's block was (`g_streamHeader` 1 once it is there, no address).
    use crate::audio::sample_data::{self as data, Buffer};
    if AUDIO.stream_header.get() == 0 {
        data::take(Buffer::Header, HEADER_LEN);
        crate::memory::count_owned(&AUDIO.stream_header_memory, HEADER_LEN);
        AUDIO.stream_header.set(1);
    }
    data::with_buffer(Buffer::Header, |b| b[..HEADER_LEN as usize].fill(0));
    if !crate::audio::samples::read_sample(path, Buffer::Header, HEADER_LEN, &AUDIO.sample_file)? {
        return failed(s);
    }
    let Some(h) = crate::files::handle::open_by_mode_at(path, 0)? else {
        return failed(s);
    };
    s.file = h;
    s.open = 1;
    let f = AUDIO.sample_file.get();
    crate::files::handle::seek(h, data::offset(f.data)? as i32, 0);
    describe_wave_format(f.format, f.rate)?;
    AUDIO.buffer_description.set(BufferDescription {
        size: 0x14,
        flags: 0x80,
        bytes: s.bytes,
        reserved: 0,
        format: 0,
    });
    s.buffer = crate::platform_sound::create_sound_buffer(
        0x80,
        s.bytes,
        Some(crate::audio::describe_wave_format::platform_format()),
    );
    crate::platform_sound::set_volume(s.buffer, volume as i32);
    // Converted: `rc2k_core::audio::streams::AudioStream::start_ring`.
    s.start_ring(f.format);
    if crate::platform_sound::play(s.buffer, 1) != 0 {
        return failed(s);
    }
    s.playing = 1;
    Ok(true)
}

/// `audio::feed_stream`: the bytes the play cursor moved over refilled, through the locked
/// parts (`g_lockedParts`). False for the carry (the player stopped).
pub fn feed_stream(s: &mut AudioStream) -> Result<bool, Refused> {
    not_a3d("A3D (0x40D0C0)")?;
    let failed = |s: &mut AudioStream| stop_stream(s).map(|_| false);
    if s.buffer == 0 {
        return failed(s);
    }
    let b = s.buffer;
    // Converted: `rc2k_core::audio::streams::AudioStream::moved` and `advance`.
    let Ok((cursor, _)) = crate::platform_sound::current_position(b) else {
        return failed(s);
    };
    let Some(moved) = s.moved(cursor) else {
        return Ok(true);
    };
    let Ok(parts) = crate::platform_sound::lock(b, s.write_at, moved, 0) else {
        return failed(s);
    };
    for (cell, part) in AUDIO.locked_parts.iter().zip(parts) {
        cell.set(part);
    }
    s.advance(moved);
    // The parts read outside the buffer's lock, then copied in as far as they were
    // written (a short read without silence leaves the rest as it was).
    let [p1, n1, p2, n2] = parts;
    let mut first = vec![0; if p1 != 0 { n1 as usize } else { 0 }];
    let mut second = vec![0; if p2 != 0 { n2 as usize } else { 0 }];
    let first_written = written(fill_stream_part(s, &mut first), first.len());
    let second_written = written(fill_stream_part(s, &mut second), second.len());
    let copied = crate::platform_sound::with_parts(b, parts, |a, c| {
        a[..first_written].copy_from_slice(&first[..first_written]);
        c[..second_written].copy_from_slice(&second[..second_written]);
    });
    if copied.is_err() || crate::platform_sound::unlock(b, parts) != 0 {
        return failed(s);
    }
    Ok(true)
}
