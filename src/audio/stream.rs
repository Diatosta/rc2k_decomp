//! The music stream: a WAV file played through a looping DirectSound buffer that is refilled from
//! the file each turn. Two players share the code (the sound stream, command 9, and the music
//! stream, command 0xB). `audio::start_stream` reads the file's head, opens the file at its data,
//! makes a buffer of `bytes` and plays it looping, with the write place `step` ahead of the play
//! cursor. `audio::feed_stream` writes into the buffer what the play cursor has moved over since
//! the last turn (at most `step`), read from the file; after the file's end, silence, `tail`
//! bytes of it, then the player stops.

use crate::audio::own::AudioOwn;
use crate::errors::{BadInput, Refused};

pub const HEADER_LEN: u32 = 0xC00;

pub use rc2k_core::audio::streams::AudioStream;

fn volume_of(state: &AudioOwn, index: u32) -> Result<u32, Refused> {
    Ok(rc2k_core::audio::levels::stream_volume(index, |n| {
        crate::audio::buffer_settings::volumes(state, n)
    })?)
}

/// What `fill_stream_part` did: `None`, nothing (no place or no bytes); else the bytes read and
/// whether the rest was filled with silence.
pub type Filled = Option<(u32, bool)>;

/// `into`'s bytes (a locked part's) filled from the file; what is not there becomes silence (0, or
/// 0x80 for eight bits), and a read that found nothing uses up `tail` (the player stops when it
/// goes below 0). An empty `into` is no part.
fn fill_stream_part(s: &mut AudioStream, into: &mut [u8]) -> Filled {
    if into.is_empty() {
        return None;
    }
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

/// Not playing; the buffer stopped and released; the file closed.
pub fn stop_stream(s: &mut AudioStream) -> Result<(), Refused> {
    let (buffer, file) = s.stop();
    if let Some(b) = buffer {
        crate::platform::sound::stop(b);
        crate::platform::sound::release(b);
    }
    if let Some(f) = file {
        crate::files::handle::close(f);
    }
    Ok(())
}

/// `SetVolume` with the player's volume.
pub(super) fn set_stream_volume(state: &AudioOwn, s: &AudioStream) -> Result<(), Refused> {
    if s.buffer == 0 {
        return Err(BadInput("a stream without its buffer").into());
    }
    let v = volume_of(state, s.volume)?;
    crate::platform::sound::set_volume(s.buffer, v as i32);
    Ok(())
}

/// The file whose name `path` reads (when it gets there) opened and played. False when the player
/// stopped.
pub fn start_stream(
    state: &AudioOwn,
    s: &mut AudioStream,
    path: impl FnOnce() -> Result<Vec<u8>, crate::errors::BadInput>,
) -> Result<bool, Refused> {
    use crate::audio::describe_wave_format::describe_wave_format;
    use crate::audio::make_buffer::BufferDescription;
    // Checked before anything is done: the volume it sets.
    let volume = volume_of(state, s.volume)?;
    let failed = |s: &mut AudioStream| stop_stream(s).map(|_| false);
    let ds = state.direct_sound.get();
    if ds == 0 {
        return failed(s);
    }
    // The header's buffer: Rust's (`audio::sample_data`), taken once and counted as the game's
    // block was.
    use crate::audio::sample_data::Buffer;
    let data = &state.sample_data;
    if state.stream_header.get() == 0 {
        data.take(Buffer::Header, HEADER_LEN);
        crate::memory::count_owned(&state.stream_header_memory, HEADER_LEN);
        state.stream_header.set(1);
    }
    data.with_buffer(Buffer::Header, |b| b[..HEADER_LEN as usize].fill(0));
    let path_text = path()?;
    if !crate::audio::samples::read_sample(
        state,
        &path_text,
        Buffer::Header,
        HEADER_LEN,
        &state.sample_file,
    )? {
        return failed(s);
    }
    let Some(h) = crate::files::handle::open_by_mode(&path_text, 0)? else {
        return failed(s);
    };
    s.file = h;
    s.open = 1;
    let f = state.sample_file.get();
    crate::files::handle::seek(h, data.offset(f.data)? as i32, 0);
    describe_wave_format(state, f.format, f.rate)?;
    state.buffer_description.set(BufferDescription {
        size: 0x14,
        flags: 0x80,
        bytes: s.bytes,
        reserved: 0,
        format: 0,
    });
    s.buffer = crate::platform::sound::create_sound_buffer(
        0x80,
        s.bytes,
        Some(crate::audio::describe_wave_format::platform_format(state)),
    );
    crate::platform::sound::set_volume(s.buffer, volume as i32);
    s.start_ring(f.format);
    if crate::platform::sound::play(s.buffer, 1) != 0 {
        return failed(s);
    }
    s.playing = 1;
    Ok(true)
}

/// The bytes the play cursor moved over refilled, through the locked parts. False when the player
/// stopped.
pub fn feed_stream(state: &AudioOwn, s: &mut AudioStream) -> Result<bool, Refused> {
    let failed = |s: &mut AudioStream| stop_stream(s).map(|_| false);
    if s.buffer == 0 {
        return failed(s);
    }
    let b = s.buffer;
    let Ok((cursor, _)) = crate::platform::sound::current_position(b) else {
        return failed(s);
    };
    let Some(moved) = s.moved(cursor) else {
        return Ok(true);
    };
    let Ok(parts) = crate::platform::sound::lock(b, s.write_at, moved, 0) else {
        return failed(s);
    };
    for (cell, part) in state.locked_parts.iter().zip(parts) {
        cell.set(part);
    }
    s.advance(moved);
    // The parts read outside the buffer's lock, then copied in as far as they were written (a short
    // read without silence leaves the rest as it was).
    let [p1, n1, p2, n2] = parts;
    let mut first = vec![0; if p1 != 0 { n1 as usize } else { 0 }];
    let mut second = vec![0; if p2 != 0 { n2 as usize } else { 0 }];
    let first_written = written(fill_stream_part(s, &mut first), first.len());
    let second_written = written(fill_stream_part(s, &mut second), second.len());
    let copied = crate::platform::sound::with_parts(b, parts, |a, c| {
        a[..first_written].copy_from_slice(&first[..first_written]);
        c[..second_written].copy_from_slice(&second[..second_written]);
    });
    if copied.is_err() || crate::platform::sound::unlock(b, parts) != 0 {
        return failed(s);
    }
    Ok(true)
}
