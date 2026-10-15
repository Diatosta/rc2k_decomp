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
use crate::errors::{BadInput, Refused};
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

unsafe fn word(at: u32) -> u32 {
    unsafe { (at as *const u16).read_unaligned() as u32 }
}

/// The flag a table of two {flag, value} entries has for `value`.
unsafe fn flag_for(table: usize, value: u32) -> Option<u32> {
    (0..2u32).find_map(|n| unsafe {
        let entry = game_addr(table) as u32 + n * 8;
        (dword(entry + 4) == value).then(|| dword(entry))
    })
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
        let mut n = read.bytes_read.unwrap_or(0) as i32;
        let mut p = buffer;
        let mut raw = true;
        if dword(p) == 0x4646_4952 && dword(p + 8) == 0x4556_4157 {
            raw = false;
            p += 12;
            n -= 12;
            if n <= 0 {
                return Ok(false);
            }
            loop {
                let (id, len) = (dword(p), dword(p + 4));
                p += 8;
                n -= 8;
                if n <= 0 {
                    return Ok(false);
                }
                if id == 0x2074_6D66 {
                    if word(p) != 1 {
                        return Ok(false);
                    }
                    f.channels = word(p + 2);
                    if len == 0xC {
                        f.rate = word(p + 4);
                        f.bits = word(p + 0xA);
                    } else {
                        f.rate = dword(p + 4);
                        f.bits = word(p + 0xE);
                    }
                } else if id == 0x6174_6164 {
                    if len <= n as u32 {
                        n = len as i32;
                    }
                    break;
                }
                p = p.wrapping_add(len);
                n = n.wrapping_sub(len as i32);
                if n <= 0 {
                    return Ok(false);
                }
            }
        }
        let _ = raw;
        let length = (n as u32) & !3;
        f.data = p;
        f.length = length;
        if !(4..=0x1000_0000).contains(&length) {
            return Ok(false);
        }
        let Some(bits) = flag_for(SAMPLE_BITS_FLAGS_RVA, f.bits) else {
            return Ok(false);
        };
        let Some(channels) = flag_for(SAMPLE_CHANNEL_FLAGS_RVA, f.channels) else {
            return Ok(false);
        };
        f.format = bits | channels;
        f.rate = f.rate.clamp(100, 200_000);
        let frame_bytes = f.bits.wrapping_mul(f.channels) >> 3;
        f.frame_bytes = frame_bytes;
        if frame_bytes == 0 {
            return Ok(false);
        }
        f.frames = length / frame_bytes;
        Ok(true)
    }
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
    (0..count)
        .map(|n| voices + n * std::mem::size_of::<Voice>() as u32)
        .find(|&at| unsafe { voice(at).data_key } == record.data_key)
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
    unsafe {
        let record = *(record_at as *const SampleRecord);
        if shared != 0 {
            // The record's chain, to find its end, before anything is written.
            if record.first_voice == 0 {
                return Err(BadInput("a shared voice for a record without voices").into());
            }
            let mut last = record.first_voice;
            let mut steps = 0;
            while last != at && voice(last).next_copy != 0 {
                last = voice(last).next_copy;
                steps += 1;
                if steps > VOICES {
                    return Err(BadInput("a record's voices without their end").into());
                }
            }
            let s = *voice(shared);
            let v = voice(at);
            v.data = s.data;
            v.length = s.length;
            v.format = s.format;
            v.frames = s.frames;
            v.own_frequency = s.own_frequency;
            v.source = shared;
            if last != at {
                voice(last).next_copy = at;
            }
            v.next_copy = 0;
            v.data_key = 0;
        } else {
            let v = voice(at);
            v.data = file.data;
            v.length = file.length;
            v.format = file.format;
            v.frames = file.frames;
            v.own_frequency = file.rate;
            v.source = 0;
            v.next_copy = 0;
            v.data_key = record.data_key;
        }
        let v = voice(at);
        v.flags = if sound_mode & 0x1_0000 != 0 { 1 } else { 2 };
        v.sound = record.number;
        v.kind = record.voice_kind;
        v.scale = record.scale;
        v.priority = record.priority as i32;
        v.volume = record.volume;
        v.pan = record.pan as u8 as i8 as i32;
        v.pan_sign = 0;
        v.frequency = if record.frequency == OWN_FREQUENCY {
            v.own_frequency
        } else {
            record.frequency
        };
        v.place = record.place;
        v.min_distance = record.min_distance;
        v.volume_factor = if record.volume_factor != 0 {
            dword(record.volume_factor)
        } else {
            0x40
        };
        v.state = 0;
        v.dirty |= 0xF;
        v.channel = 0;
        v.next = 0;
    }
    Ok(())
}

/// `audio::sample_mode`: the mode a voice's sample is prepared for.
pub fn sample_mode(mode: u32, record: Option<&SampleRecord>) -> u32 {
    let Some(r) = record else { return mode };
    let at_least = |n: u32| {
        if mode & SHIFT < n {
            mode - (mode & SHIFT) + n
        } else {
            mode
        }
    };
    if r.number == 0x1C4 {
        mode & SHIFT
    } else {
        match r.group {
            1 | 2 => at_least(1),
            3 => at_least(2),
            _ => mode,
        }
    }
}

/// `audio::prepare_sample`: the voice's data, in place, scaled by `scale` / 64, halved
/// until its shift is the mode's, and made eight bits if the mode asks.
fn stereo_doubles(frames: u32, format: u32) -> u32 {
    if format & MONO != 0 {
        frames
    } else {
        frames.wrapping_add(frames)
    }
}

/// The counts `prepare_sample`'s loops would run with: a count of 0 runs the original's
/// `dec ecx; jnz` through 2^32 steps.
fn counts_sound(v: &Voice, mode: u32) -> bool {
    if v.scale != 0x40 && stereo_doubles(v.frames, v.format) == 0 {
        return false;
    }
    let (mut frames, mut length, mut format) = (v.frames, v.length, v.format);
    if mode & 3 != 0 {
        while v.own_frequency >= 0xC8 && length >= 8 {
            if frames >> 1 == 0 {
                return false;
            }
            format = format.wrapping_add(1);
            length >>= 1;
            frames >>= 1;
            if (format ^ mode) & SHIFT == 0 {
                break;
            }
        }
    }
    !(mode & EIGHT_BITS != 0
        && format & EIGHT_BITS == 0
        && length >= 8
        && stereo_doubles(frames, format) == 0)
}

pub unsafe fn prepare_sample(v: &mut Voice, mode: u32) -> Result<(), Refused> {
    unsafe {
        if v.data == 0 || v.length == 0 || v.data_key == 0 {
            return Ok(());
        }
        if !counts_sound(v, mode) {
            return Err(BadInput("a count of 0 for one of the sample's loops").into());
        }
        if v.scale != 0x40 {
            let scale = v.scale as i32;
            let mut count = stereo_doubles(v.frames, v.format);
            let (mut from, mut to) = (v.data, v.data);
            if v.format & EIGHT_BITS != 0 {
                loop {
                    let s = ((*(from as *const u8) as i32 - 0x80).wrapping_mul(scale) >> 6)
                        .clamp(-0x80, 0x7F);
                    from += 1;
                    *(to as *mut u8) = (s + 0x80) as u8;
                    to += 1;
                    count = count.wrapping_sub(1);
                    if count == 0 {
                        break;
                    }
                }
            } else {
                loop {
                    let s = (((from as *const i16).read_unaligned() as i32).wrapping_mul(scale)
                        >> 6)
                        .clamp(-0x8000, 0x7FFF);
                    from += 2;
                    (to as *mut i16).write_unaligned(s as i16);
                    to += 2;
                    count = count.wrapping_sub(1);
                    if count == 0 {
                        break;
                    }
                }
            }
        }
        if mode & 3 != 0 {
            loop {
                if v.own_frequency < 0xC8 || v.length < 8 {
                    break;
                }
                let mut count = v.frames >> 1;
                let (mut from, mut to) = (v.data, v.data);
                let b = |at: u32| *(at as *const u8) as u32;
                let w = |at: u32| (at as *const i16).read_unaligned() as i32;
                loop {
                    match (v.format & EIGHT_BITS != 0, v.format & MONO != 0) {
                        (true, true) => {
                            *(to as *mut u8) = ((b(from) + b(from + 1)) >> 1) as u8;
                            from += 2;
                            to += 1;
                        }
                        (true, false) => {
                            *(to as *mut u8) = ((b(from) + b(from + 2)) >> 1) as u8;
                            *((to + 1) as *mut u8) = ((b(from + 1) + b(from + 3)) >> 1) as u8;
                            from += 4;
                            to += 2;
                        }
                        (false, true) => {
                            (to as *mut i16).write_unaligned(((w(from) + w(from + 2)) >> 1) as i16);
                            from += 4;
                            to += 2;
                        }
                        (false, false) => {
                            (to as *mut i16).write_unaligned(((w(from) + w(from + 4)) >> 1) as i16);
                            ((to + 2) as *mut i16)
                                .write_unaligned(((w(from + 2) + w(from + 6)) >> 1) as i16);
                            from += 8;
                            to += 4;
                        }
                    }
                    count = count.wrapping_sub(1);
                    if count == 0 {
                        break;
                    }
                }
                v.format = v.format.wrapping_add(1);
                v.length >>= 1;
                v.frames >>= 1;
                if (v.format ^ mode) & SHIFT == 0 {
                    break;
                }
            }
        }
        if mode & EIGHT_BITS != 0 && v.format & EIGHT_BITS == 0 && v.length >= 8 {
            let mut count = stereo_doubles(v.frames, v.format);
            let (mut from, mut to) = (v.data, v.data);
            loop {
                *(to as *mut u8) = (*((from + 1) as *const u8)).wrapping_add(0x80);
                from += 2;
                to += 1;
                count = count.wrapping_sub(1);
                if count == 0 {
                    break;
                }
            }
            v.format = (v.format & 0xFFFF_FF0F) | EIGHT_BITS;
            v.length >>= 1;
        }
    }
    Ok(())
}
