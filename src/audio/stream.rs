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
use crate::errors::{BadInput, Refused};
use crate::framework::rva;

pub const SOUND_STREAM_RVA: usize = rva(0x506960); // g_soundStream
pub const MUSIC_STREAM_RVA: usize = rva(0x506990); // g_musicStream
pub const NEXT_TUNE_RVA: usize = rva(0x50698C); // g_nextTune
pub const NO_MUSIC_RVA: usize = rva(0x5069BC); // g_noMusic
pub const MUSIC_PATH_RVA: usize = rva(0x5069C0); // g_musicPath ";3files\music\00.wav"
pub const HEADER_RVA: usize = rva(0x5069D8); // g_streamHeader
pub const HEADER_MEMORY_RVA: usize = rva(0x5069DC); // g_streamHeaderMemory (a MemoryList)
pub const TUNE_TURN_RVA: usize = rva(0x5180F8); // g_tuneTurn
pub const HEADER_LEN: u32 = 0xC00;
/// `GetCurrentPosition`, `Lock`, `Unlock` of a DirectSound buffer.
pub const GET_POSITION: u32 = 0x10;
pub const LOCK: u32 = 0x2C;
pub const UNLOCK: u32 = 0x4C;

pub use rc2k_core::audio::streams::AudioStream;

pub unsafe fn stream<'a>(at: u32) -> &'a mut AudioStream {
    unsafe { &mut *(at as *mut AudioStream) }
}

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

/// `audio::fill_stream_part`: `bytes` at `at` from the file; what is not there becomes
/// silence (0, or 0x80 for eight bits), and a read that found nothing uses up `tail`
/// (the player stops when it goes below 0).
pub unsafe fn fill_stream_part(s: &mut AudioStream, at: u32, bytes: u32) -> Filled {
    if at == 0 || bytes == 0 {
        return None;
    }
    // Converted: `rc2k_core::audio::streams::AudioStream::after_read`.
    let (read, _) = unsafe { crate::files::handle::read(s.file, at as *mut u8, bytes) };
    let Some(quiet) = s.after_read(bytes, read) else {
        return Some((read, false));
    };
    unsafe { std::ptr::write_bytes((at + read) as *mut u8, quiet, (bytes - read) as usize) };
    Some((read, true))
}

/// The registers `fill_stream_part` leaves: eax, ecx, edi (its one caller keeps its own).
pub fn fill_left(r: &mut crate::framework::Regs, at: u32, bytes: u32, s: &AudioStream, f: Filled) {
    match f {
        None => {}
        Some((read, false)) => r.eax = read,
        Some((_, true)) => {
            r.eax = u32::from_ne_bytes([s.silence(); 4]);
            r.ecx = 0;
            r.edi = at + bytes;
        }
    }
}

/// `audio::stop_stream`: not playing; the buffer stopped and released; the file closed.
pub unsafe fn stop_stream(s: &mut AudioStream) -> Result<(), Refused> {
    not_a3d("A3D (0x40D1D0)")?;
    s.playing = 0;
    unsafe {
        if s.buffer != 0 {
            crate::audio::com::call(s.buffer, crate::audio::buffer_play::STOP);
            crate::audio::com::call(s.buffer, crate::audio::com::RELEASE);
            s.buffer = 0;
        }
        if s.open == 1 {
            crate::files::handle::close(s.file);
            s.open = 0;
        }
    }
    Ok(())
}

/// `audio::set_stream_volume`: `SetVolume` with the player's volume.
pub unsafe fn set_stream_volume(s: &AudioStream) -> Result<(), Refused> {
    not_a3d("A3D (0x40D247)")?;
    if s.buffer == 0 {
        return Err(BadInput("a stream without its buffer").into());
    }
    let v = volume_of(s.volume)?;
    unsafe { crate::audio::com::call1(s.buffer, crate::audio::buffer_settings::SET_VOLUME, v) };
    Ok(())
}

/// Where `start_stream` keeps its things.
#[derive(Clone, Copy)]
pub struct StartVars {
    pub direct_sound: u32,
    pub header: u32,
    pub header_memory: u32,
}

impl StartVars {
    pub fn game() -> Self {
        let a = |r| crate::framework::game_addr(r) as u32;
        Self {
            direct_sound: a(crate::audio::make_buffer::DIRECT_SOUND_RVA),
            header: a(HEADER_RVA),
            header_memory: a(HEADER_MEMORY_RVA),
        }
    }
}

/// `audio::start_stream`: the file at `path` opened and played. False for the carry (the
/// player stopped).
pub unsafe fn start_stream(
    s: &mut AudioStream,
    path: *const u8,
    v: &StartVars,
) -> Result<bool, Refused> {
    use crate::audio::describe_wave_format::{WAVE_FORMAT_RVA, describe_wave_format};
    use crate::audio::make_buffer::{BufferDescription, DESCRIPTION_RVA};
    use crate::audio::samples::{SAMPLE_FILE_RVA, SampleFile};
    use crate::audio::voices::{read, write};
    use crate::framework::game_addr;
    not_a3d("A3D (0x40CF6B)")?;
    // Checked before anything is done: the volume it sets.
    let volume = volume_of(s.volume)?;
    unsafe {
        let failed = |s: &mut AudioStream| stop_stream(s).map(|_| false);
        let ds = read(v.direct_sound);
        if ds == 0 {
            return failed(s);
        }
        let mut header = read(v.header);
        if header == 0 {
            header = crate::memory::allocate::allocate(
                v.header_memory as *mut crate::memory::MemoryList,
                HEADER_LEN,
            )? as u32;
            write(v.header, header);
        }
        std::ptr::write_bytes(header as *mut u8, 0, HEADER_LEN as usize);
        let file = game_addr(SAMPLE_FILE_RVA) as *mut SampleFile;
        if !crate::audio::samples::read_sample(path, header, HEADER_LEN, file)? {
            return failed(s);
        }
        let Some(h) = crate::files::handle::open_by_mode(path, 0)? else {
            return failed(s);
        };
        s.file = h;
        s.open = 1;
        let f = *file;
        crate::files::handle::seek(h, f.data.wrapping_sub(header) as i32, 0);
        describe_wave_format(f.format, f.rate)?;
        let description = game_addr(DESCRIPTION_RVA) as *mut BufferDescription;
        description.write_unaligned(BufferDescription {
            size: 0x14,
            flags: 0x80,
            bytes: s.bytes,
            reserved: 0,
            format: game_addr(WAVE_FORMAT_RVA) as u32,
        });
        let create: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(ds, 0x0C));
        if create(ds, description as u32, &mut s.buffer as *mut u32 as u32, 0) != 0 {
            return failed(s);
        }
        crate::audio::com::call1(s.buffer, crate::audio::buffer_settings::SET_VOLUME, volume);
        s.write_at = s.step;
        s.play_cursor = 0;
        s.tail = s.step as i32;
        s.format = f.format;
        let play: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 = std::mem::transmute(
            crate::audio::com::method(s.buffer, crate::audio::buffer_play::PLAY),
        );
        if play(s.buffer, 0, 0, 1) != 0 {
            return failed(s);
        }
        s.playing = 1;
        Ok(true)
    }
}

/// `audio::feed_stream`: the bytes the play cursor moved over refilled, through the locked
/// parts at `parts` (`g_lockedParts`). False for the carry (the player stopped).
pub unsafe fn feed_stream(s: &mut AudioStream, parts: u32) -> Result<bool, Refused> {
    use crate::audio::voices::read;
    not_a3d("A3D (0x40D0C0)")?;
    unsafe {
        let failed = |s: &mut AudioStream| stop_stream(s).map(|_| false);
        if s.buffer == 0 {
            return failed(s);
        }
        let b = s.buffer;
        // Converted: `rc2k_core::audio::streams::AudioStream::moved` and `advance`.
        let position: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(b, GET_POSITION));
        let mut cursor = s.play_cursor;
        let failed_position = position(b, &mut cursor as *mut u32 as u32, 0) != 0;
        if failed_position {
            // The original's call wrote nothing it did not have.
            s.play_cursor = cursor;
            return failed(s);
        }
        let Some(moved) = s.moved(cursor) else {
            return Ok(true);
        };
        let lock: unsafe extern "system" fn(u32, u32, u32, u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(b, LOCK));
        if lock(
            b,
            s.write_at,
            moved,
            parts,
            parts + 4,
            parts + 8,
            parts + 12,
            0,
        ) != 0
        {
            return failed(s);
        }
        s.advance(moved);
        fill_stream_part(s, read(parts), read(parts + 4));
        fill_stream_part(s, read(parts + 8), read(parts + 12));
        let [p1, n1, p2, n2] = (parts as *const [u32; 4]).read_unaligned();
        let unlock: unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(b, UNLOCK));
        if unlock(b, p1, n1, p2, n2) != 0 {
            return failed(s);
        }
        Ok(true)
    }
}

// ---- for the self-tests ----

/// A WAV file of the test's in `var`: `DATA_LEN` bytes of data after a 0x2C-byte head.
pub const TEST_FILE: &str = "var\\rc2k_hook_stream_test.wav";
pub const TEST_PATH: &[u8] = b"var\\rc2k_hook_stream_test.wav\0";
pub const MISSING_PATH: &[u8] = b"var\\rc2k_hook_no_such_stream.wav\0";
pub const DATA_LEN: u32 = 0x300;
pub const DATA_AT: u32 = 0x2C;

/// Writes the test file: 16-bit stereo, or 8-bit mono with `eight`.
pub fn make_test_file(eight: bool) -> bool {
    use crate::audio::read_sample::{fmt, riff};
    let data: Vec<u8> = (0..DATA_LEN).map(|i| (i * 13 + 5) as u8).collect();
    let format = if eight {
        fmt(1, 1, 11025, 8)
    } else {
        fmt(1, 2, 22050, 16)
    };
    std::fs::write(
        TEST_FILE,
        riff(b"WAVE", &[(b"fmt ", &format), (b"data", &data)]),
    )
    .is_ok()
}

pub fn remove_test_file() {
    let _ = std::fs::remove_file(TEST_FILE);
}

/// The test file opened at `offset` of its data (None: it cannot be).
pub fn open_test_file(offset: u32) -> Option<u32> {
    unsafe {
        let h = crate::files::handle::open_by_mode(TEST_PATH.as_ptr(), 0).ok()??;
        crate::files::handle::seek(h, (DATA_AT + offset) as i32, 0);
        Some(h)
    }
}

/// Where an open file is (None: it is closed).
pub fn position(file: u32) -> Option<u32> {
    unsafe { crate::files::handle::seek(file, 0, 1) }
}

/// A player with the file field cleared (the handles of two runs differ).
pub fn without_file(s: &AudioStream) -> AudioStream {
    AudioStream { file: 0, ..*s }
}
