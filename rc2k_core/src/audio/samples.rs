//! The audio thread's sample files and formats (converted 2026-10-20 from
//! `audio::read_sample` 0x40B21B, `audio::describe_wave_format` 0x40A6A5 and
//! `audio::volume_index` 0x40B8D1).

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// A table of two (flag, value) pairs: the format bits for 8 and 16 bits, for one and two
/// channels.
pub type FormatBitTable = [(u32, u32); 2];

/// The value a table has for `flag`.
pub fn value_for(table: &FormatBitTable, flag: u32) -> Option<u32> {
    table.iter().find(|e| e.0 == flag).map(|e| e.1)
}

/// The flag a table has for `value`.
pub fn flag_for(table: &FormatBitTable, value: u32) -> Option<u32> {
    table.iter().find(|e| e.1 == value).map(|e| e.0)
}

/// The first index of the 0x40 volumes whose volume is at least `volume`, else 0x40.
pub fn volume_index(volume: i32, table: &[i32; 0x40]) -> u32 {
    table.iter().position(|&v| volume <= v).unwrap_or(0x40) as u32
}

/// A PCM `WAVEFORMATEX`'s figures.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WaveFormat {
    pub channels: u16,
    pub samples_per_second: u32,
    pub bytes_per_second: u32,
    pub block_align: u16,
    pub bits: u16,
}

/// What `wave_format` cannot find (the original's fatal errors 0x80A28541, 0x80A28542).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatError {
    Channels,
    Bits,
}

/// The wave format of a sample format word at `frequency`: the rate the frequency shifted
/// by the word's four low bits, channels by bits 8 .. 11, bits by 4 .. 7.
pub fn wave_format(
    format: u32,
    frequency: u32,
    bits_table: &FormatBitTable,
    channel_table: &FormatBitTable,
) -> Result<WaveFormat, FormatError> {
    let channels = value_for(channel_table, format & 0xF00).ok_or(FormatError::Channels)?;
    let bits = value_for(bits_table, format & 0xF0).ok_or(FormatError::Bits)?;
    let rate = frequency >> (format & 0xF);
    let block = bits.wrapping_mul(channels as u16 as u32) >> 3;
    Ok(WaveFormat {
        channels: channels as u16,
        samples_per_second: rate,
        bytes_per_second: block.wrapping_mul(rate),
        block_align: block as u16,
        bits: bits as u16,
    })
}

/// A sample as read (`g_sampleFile`); `data` is an offset from the work buffer's start.
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

impl SampleFile {
    /// What a read begins with: one channel, 22050 Hz, sixteen bits.
    pub fn begin(&mut self) {
        self.channels = 1;
        self.rate = 22050;
        self.bits = 16;
    }
}

const RIFF: u32 = 0x4646_4952;
const WAVE: u32 = 0x4556_4157;
const FMT: u32 = 0x2074_6D66;
const DATA: u32 = 0x6174_6164;

/// The `read` bytes of a sample file in `file` (the work buffer: reads go where the
/// original's do, past what was read too) parsed into `f`, each field written where the
/// original writes it; true when a sample was found. RIFF / WAVE: chunks walked (no
/// padding), `fmt ` must be PCM and gives channels, rate and bits (a 12-byte chunk as
/// words), `data` the data (at most what was read); a walk that runs out finds nothing.
/// Anything else is raw data, all of it. Then the length rounded down to four and
/// 4 ..= 0x10000000, bits and channels by their tables, the rate held to 100 ..= 200,000,
/// bytes a frame not 0. Refused: a read the source cannot answer.
pub fn parse(
    file: &dyn Bytes,
    read: u32,
    f: &mut SampleFile,
    bits_table: &FormatBitTable,
    channel_table: &FormatBitTable,
) -> Result<bool, Invalid> {
    let past = Invalid("a sample file read past the work buffer");
    let dword = |at: u32| file.u32_le(at as usize).ok_or(past);
    let word = |at: u32| -> Result<u32, Invalid> {
        Ok(u16::from_le_bytes([
            file.byte(at as usize).ok_or(past)?,
            file.byte(at.wrapping_add(1) as usize).ok_or(past)?,
        ]) as u32)
    };
    let mut n = read as i32;
    let mut p = 0u32;
    if dword(0)? == RIFF && dword(8)? == WAVE {
        p = 12;
        n -= 12;
        if n <= 0 {
            return Ok(false);
        }
        loop {
            let (id, len) = (dword(p)?, dword(p.wrapping_add(4))?);
            p = p.wrapping_add(8);
            n -= 8;
            if n <= 0 {
                return Ok(false);
            }
            if id == FMT {
                if word(p)? != 1 {
                    return Ok(false);
                }
                f.channels = word(p.wrapping_add(2))?;
                if len == 0xC {
                    f.rate = word(p.wrapping_add(4))?;
                    f.bits = word(p.wrapping_add(0xA))?;
                } else {
                    f.rate = dword(p.wrapping_add(4))?;
                    f.bits = word(p.wrapping_add(0xE))?;
                }
            } else if id == DATA {
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
    let length = (n as u32) & !3;
    f.data = p;
    f.length = length;
    if !(4..=0x1000_0000).contains(&length) {
        return Ok(false);
    }
    let Some(bits) = flag_for(bits_table, f.bits) else {
        return Ok(false);
    };
    let Some(channels) = flag_for(channel_table, f.channels) else {
        return Ok(false);
    };
    f.format = bits | channels;
    f.rate = f.rate.clamp(100, 200_000);
    f.frame_bytes = f.bits.wrapping_mul(f.channels) >> 3;
    if f.frame_bytes == 0 {
        return Ok(false);
    }
    f.frames = length / f.frame_bytes;
    Ok(true)
}

/// A volume or rate meaning the sample record's own.
pub const OWN: u32 = 0xFFFF_FF91;
/// A rate meaning the sample's voice's own frequency.
pub const VOICE_RATE: u32 = 0xFFFF_FF90;

/// What `settings` and `duration_ms` read of a sample record and its first voice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Loaded {
    /// The record's voice kind (1: looping).
    pub voice_kind: u32,
    /// The record's volume and frequency.
    pub volume: u32,
    pub frequency: u32,
    /// The voice's own frequency, format (its shift in the low four bits) and frames.
    pub own_frequency: u32,
    pub format: u32,
    pub frames: u32,
}

/// A rate as asked: `OWN` the record's (and that may be `VOICE_RATE`), `VOICE_RATE` the
/// voice's, held to 100 ..= 200,000 (signed).
pub fn rate_of(rate: u32, s: &Loaded) -> u32 {
    let rate = if rate == OWN { s.frequency } else { rate };
    let rate = if rate == VOICE_RATE {
        s.own_frequency
    } else {
        rate
    };
    (rate as i32).clamp(100, 200_000) as u32
}

/// A sound's volume and rate (`sound::sample_settings` 0x40BA71): for a sample not loaded
/// (`None`) 0 and 44100; else the volume (`OWN` the record's) held to 0 ..= 0x40
/// (signed), and `rate_of`.
pub fn settings(volume: u32, rate: u32, sample: Option<&Loaded>) -> (u32, u32) {
    let Some(s) = sample else {
        return (0, 44100);
    };
    let volume = if volume == OWN { s.volume } else { volume };
    ((volume as i32).clamp(0, 0x40) as u32, rate_of(rate, s))
}

/// How long a sample plays at a rate, in ms (`sound::sample_duration` 0x40BA0B): 0 not
/// loaded, 0x7FFFFFFF looping, else frames x 1000 (the low 32 bits) over the rate shifted
/// by the voice's shift. Refused: a rate the shift makes 0 (the original's `div` traps).
pub fn duration_ms(rate: u32, sample: Option<&Loaded>) -> Result<u32, Invalid> {
    let Some(s) = sample else {
        return Ok(0);
    };
    if s.voice_kind == 1 {
        return Ok(0x7FFF_FFFF);
    }
    let rate = rate_of(rate, s) >> (s.format & 0xF);
    if rate == 0 {
        return Err(Invalid(
            "a rate the shift makes 0: the original divides by it",
        ));
    }
    Ok(s.frames.wrapping_mul(1000) / rate)
}

/// The mode bits of a sample's shift (its rate halved so many times).
pub const SHIFT: u32 = 0xF;
/// A format bit: eight bits a value.
pub const EIGHT_BITS: u32 = 0x10;
/// A format bit: one value a frame.
pub const MONO: u32 = 0x100;

/// The mode a voice's sample is prepared for (`audio::sample_mode` 0x40B0EF): the sound
/// mode, with a record (its number and group) at least shift 1 for groups 1 and 2, 2 for
/// group 3, and only the shift for sample 0x1C4.
pub fn sample_mode(mode: u32, record: Option<(u32, u32)>) -> u32 {
    let Some((number, group)) = record else {
        return mode;
    };
    let at_least = |n: u32| {
        if mode & SHIFT < n {
            mode - (mode & SHIFT) + n
        } else {
            mode
        }
    };
    if number == 0x1C4 {
        mode & SHIFT
    } else {
        match group {
            1 | 2 => at_least(1),
            3 => at_least(2),
            _ => mode,
        }
    }
}

/// A sample's data, read and written at offsets from its start.
pub trait SampleData {
    fn get(&self, at: u32) -> Option<u8>;
    /// False past the end.
    fn set(&mut self, at: u32, value: u8) -> bool;
}

/// A borrowed slice (sized, so it can be `&mut dyn SampleData`): memory the caller owns.
impl SampleData for &mut [u8] {
    fn get(&self, at: u32) -> Option<u8> {
        (**self).get(at as usize).copied()
    }
    fn set(&mut self, at: u32, value: u8) -> bool {
        match (**self).get_mut(at as usize) {
            Some(b) => {
                *b = value;
                true
            }
            None => false,
        }
    }
}

impl SampleData for Vec<u8> {
    fn get(&self, at: u32) -> Option<u8> {
        self.as_slice().get(at as usize).copied()
    }
    fn set(&mut self, at: u32, value: u8) -> bool {
        match self.get_mut(at as usize) {
            Some(b) => {
                *b = value;
                true
            }
            None => false,
        }
    }
}

/// What `prepare` reads and changes of a voice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shape {
    pub length: u32,
    pub format: u32,
    pub frames: u32,
    /// The volume scale out of 0x40.
    pub scale: u32,
    pub own_frequency: u32,
}

/// Values a frame's worth: frames, doubled for stereo.
fn values(frames: u32, format: u32) -> u32 {
    if format & MONO != 0 {
        frames
    } else {
        frames.wrapping_add(frames)
    }
}

/// Whether every loop `prepare` would run has a count (a count of 0 runs the original's
/// `dec ecx; jnz` through 2^32 steps).
fn counts_sound(v: &Shape, mode: u32) -> bool {
    if v.scale != 0x40 && values(v.frames, v.format) == 0 {
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
        && values(frames, format) == 0)
}

/// A voice's sample made ready for `mode` in place (`audio::prepare_sample` 0x40B13C):
/// scaled by `scale` / 0x40 (held to its bits' range), halved (each pair of frames
/// averaged) until its shift is the mode's while it is long enough and its own frequency
/// at least 200, and made eight bits if the mode asks (the high byte, moved by 0x80).
/// Refused (before anything): a loop with a count of 0; a read or write past the data.
pub fn prepare(v: &mut Shape, mode: u32, data: &mut dyn SampleData) -> Result<(), Invalid> {
    if !counts_sound(v, mode) {
        return Err(Invalid("a count of 0 for one of the sample's loops"));
    }
    let past = Invalid("a sample prepared past its data");
    let get = |data: &dyn SampleData, at: u32| data.get(at).ok_or(past);
    let word = |data: &dyn SampleData, at: u32| -> Result<i32, Invalid> {
        Ok(i16::from_le_bytes([get(data, at)?, get(data, at.wrapping_add(1))?]) as i32)
    };
    let put = |data: &mut dyn SampleData, at: u32, b: u8| {
        if data.set(at, b) { Ok(()) } else { Err(past) }
    };
    let put_word = |data: &mut dyn SampleData, at: u32, w: i32| -> Result<(), Invalid> {
        let [lo, hi] = (w as i16).to_le_bytes();
        put(data, at, lo)?;
        put(data, at.wrapping_add(1), hi)
    };
    if v.scale != 0x40 {
        let scale = v.scale as i32;
        let count = values(v.frames, v.format);
        let mut n = 0u32;
        loop {
            if v.format & EIGHT_BITS != 0 {
                let s = ((get(data, n)? as i32 - 0x80).wrapping_mul(scale) >> 6).clamp(-0x80, 0x7F);
                put(data, n, (s + 0x80) as u8)?;
            } else {
                let at = n.wrapping_mul(2);
                let s = (word(data, at)?.wrapping_mul(scale) >> 6).clamp(-0x8000, 0x7FFF);
                put_word(data, at, s)?;
            }
            n = n.wrapping_add(1);
            if n == count {
                break;
            }
        }
    }
    if mode & 3 != 0 {
        while v.own_frequency >= 0xC8 && v.length >= 8 {
            let count = v.frames >> 1;
            let (mut from, mut to) = (0u32, 0u32);
            for _ in 0..count {
                let b = |data: &dyn SampleData, at: u32| -> Result<u32, Invalid> {
                    Ok(get(data, from.wrapping_add(at))? as u32)
                };
                let w = |data: &dyn SampleData, at: u32| word(data, from.wrapping_add(at));
                match (v.format & EIGHT_BITS != 0, v.format & MONO != 0) {
                    (true, true) => {
                        let s = (b(data, 0)? + b(data, 1)?) >> 1;
                        put(data, to, s as u8)?;
                        from = from.wrapping_add(2);
                        to = to.wrapping_add(1);
                    }
                    (true, false) => {
                        let (l, r) = (
                            (b(data, 0)? + b(data, 2)?) >> 1,
                            (b(data, 1)? + b(data, 3)?) >> 1,
                        );
                        put(data, to, l as u8)?;
                        put(data, to.wrapping_add(1), r as u8)?;
                        from = from.wrapping_add(4);
                        to = to.wrapping_add(2);
                    }
                    (false, true) => {
                        let s = (w(data, 0)? + w(data, 2)?) >> 1;
                        put_word(data, to, s)?;
                        from = from.wrapping_add(4);
                        to = to.wrapping_add(2);
                    }
                    (false, false) => {
                        let (l, r) = (
                            (w(data, 0)? + w(data, 4)?) >> 1,
                            (w(data, 2)? + w(data, 6)?) >> 1,
                        );
                        put_word(data, to, l)?;
                        put_word(data, to.wrapping_add(2), r)?;
                        from = from.wrapping_add(8);
                        to = to.wrapping_add(4);
                    }
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
        let count = values(v.frames, v.format);
        let mut n = 0u32;
        loop {
            let high = get(data, n.wrapping_mul(2).wrapping_add(1))?;
            put(data, n, high.wrapping_add(0x80))?;
            n = n.wrapping_add(1);
            if n == count {
                break;
            }
        }
        v.format = (v.format & 0xFFFF_FF0F) | EIGHT_BITS;
        v.length >>= 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BITS: FormatBitTable = [(0x10, 8), (0, 16)];
    const CHANNELS: FormatBitTable = [(0x100, 1), (0, 2)];

    #[test]
    fn formats_and_files() {
        let w = wave_format(0x101, 44100, &BITS, &CHANNELS).unwrap();
        assert_eq!(
            (w.channels, w.samples_per_second, w.bits, w.block_align),
            (1, 22050, 16, 2)
        );
        assert_eq!(
            wave_format(0x200, 1, &BITS, &CHANNELS),
            Err(FormatError::Channels)
        );
        let mut table = [0i32; 0x40];
        for (n, v) in table.iter_mut().enumerate() {
            *v = -10000 + n as i32 * 100;
        }
        assert_eq!(volume_index(-10000, &table), 0);
        assert_eq!(volume_index(-9950, &table), 1);
        assert_eq!(volume_index(1, &table), 0x40);
        // A WAVE file: fmt (16 bytes, stereo, 11025 Hz, 8 bits), then data of 8 bytes.
        let mut file = Vec::new();
        file.extend_from_slice(b"RIFF\0\0\0\0WAVEfmt ");
        file.extend_from_slice(&16u32.to_le_bytes());
        file.extend_from_slice(&[1, 0, 2, 0]);
        file.extend_from_slice(&11025u32.to_le_bytes());
        file.extend_from_slice(&[0; 6]);
        file.extend_from_slice(&8u16.to_le_bytes());
        file.extend_from_slice(b"data");
        file.extend_from_slice(&8u32.to_le_bytes());
        file.extend_from_slice(&[0x80; 8]);
        let mut f = SampleFile::default();
        f.begin();
        assert!(parse(&file, file.len() as u32, &mut f, &BITS, &CHANNELS).unwrap());
        assert_eq!(
            (f.data, f.length, f.format, f.rate, f.frame_bytes, f.frames),
            (44, 8, 0x10, 11025, 2, 4)
        );
        // Raw data of 3 bytes: too short.
        let mut f = SampleFile::default();
        f.begin();
        assert!(
            !parse(
                &[1u8, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                3,
                &mut f,
                &BITS,
                &CHANNELS
            )
            .unwrap()
        );
        assert_eq!((f.data, f.length), (0, 0));
    }

    #[test]
    fn prepares() {
        assert_eq!(sample_mode(0x101, Some((5, 3))), 0x102);
        assert_eq!(sample_mode(0x103, Some((0x1C4, 1))), 3);
        // Sixteen-bit mono, 8 frames, scale 0x20 (halved), shift 0 -> 1, then eight bits.
        let mut data: Vec<u8> = [100i16, 200, -100, -300, 4, 8, 0x7FFF, 0x7FFF]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let mut v = Shape {
            length: 16,
            format: MONO,
            frames: 8,
            scale: 0x20,
            own_frequency: 22050,
        };
        prepare(&mut v, MONO | 1, &mut data).unwrap();
        assert_eq!((v.format, v.length, v.frames), (MONO | 1, 8, 4));
        let words: Vec<i16> = data[..8]
            .chunks(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]))
            .collect();
        assert_eq!(words, [75, -100, 3, 0x3FFF]);
        let mut v = Shape {
            length: 16,
            format: MONO,
            frames: 8,
            scale: 0x40,
            own_frequency: 0,
        };
        prepare(&mut v, EIGHT_BITS | MONO, &mut data).unwrap();
        assert_eq!((v.format, v.length), (EIGHT_BITS | MONO, 8));
        // Past the data: refused.
        let mut v = Shape {
            length: 16,
            format: MONO,
            frames: 9,
            scale: 0x20,
            own_frequency: 0,
        };
        assert!(prepare(&mut v, MONO, &mut data).is_err());
    }

    #[test]
    fn settings_and_durations() {
        let s = Loaded {
            voice_kind: 0,
            volume: 0x50,
            frequency: VOICE_RATE,
            own_frequency: 22050,
            format: 1,
            frames: 11025,
        };
        assert_eq!(settings(OWN, OWN, Some(&s)), (0x40, 22050));
        assert_eq!(settings(5, 7, None), (0, 44100));
        assert_eq!(rate_of(0x8000_0000, &s), 100);
        assert_eq!(duration_ms(OWN, Some(&s)), Ok(1000));
        let s = Loaded { format: 0xF, ..s };
        assert!(duration_ms(100, Some(&s)).is_err());
    }
}
