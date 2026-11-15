//! A stand-in for DirectSound for the standalone program, on one SDL3 audio stream.
//!
//! The audio code calls DirectSound's methods as the functions here, on objects held as ids: a
//! device, its buffers (the primary one and the secondary ones, with duplicates sharing their
//! data), the buffers' 3D interfaces and the 3D listener. Results are DirectSound's codes, which
//! the game checks and logs. A small software mixer in SDL's audio callback plays every playing
//! buffer: its data (8 or 16 bits, mono or stereo) at its frequency, with its volume and pan, into
//! 44100 Hz 16-bit stereo; the primary buffer too, which in the Dolby mode is where the game's own
//! mixer writes, so the music (a secondary buffer) is heard there as well (fixed bug: "No music in
//! races with the Dolby mode, and the Dolby mode's quieter, harsher effects"). The primary buffer's
//! volume is the whole output's. The secondary buffers play at `SECONDARY_GAIN`: the game's own
//! mixer leaves 12 dB of headroom, which would put the effects of the Dolby mode 12 dB below the
//! other modes'; the gain makes every mode, and the music beside the Dolby mode's effects, sound
//! alike. 3D placing: a buffer's 3D interface keeps its place, its least and greatest distances and
//! its mode; the listener stays where the game leaves it (at the origin, facing +z: the game places
//! the sounds around the listener itself) with its roll-off factor. A placed buffer is heard at
//! DirectSound 3D's distance (the least distance over the distance, to the roll-off's power, held
//! at the greatest distance) and panned by its direction (equal power); its own pan is not used
//! then, as in DirectSound. Not done: Doppler, cones, EAX and the voice manager (their property
//! sets are not offered: `query_interface` answers `E_NOINTERFACE`, as a DirectSound without them).

use rc2k_core::math::portable;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

const OUT_RATE: u32 = 44100;
const E_NOINTERFACE: i32 = 0x8000_4002u32 as i32;
/// `DSERR_NODRIVER`.
const NO_DRIVER: i32 = 0x8878_0078u32 as i32;
const E_INVALIDARG: i32 = 0x8007_0057u32 as i32;
const IID_3D_BUFFER: u32 = 0x279A_FA86;
const IID_3D_LISTENER: u32 = 0x279A_FA84;
/// `DSBCAPS_PRIMARYBUFFER`, `DSBPLAY_LOOPING`, `DSBSTATUS_PLAYING` / `_LOOPING`.
const PRIMARY: u32 = 1;
const LOOPING: u32 = 1;
const STATUS_PLAYING: u32 = 1;
const STATUS_LOOPING: u32 = 4;
/// What `GetCaps` says of the device (as Wine's does: no hardware buffers).
const CAPS_FLAGS: u32 = 0xF5F;

/// A DirectSound result: `OK`, or the failure code the game logs.
pub const OK: i32 = 0;

/// A sample format (`WAVEFORMATEX`'s channels, bits and rate).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Format {
    pub channels: u32,
    pub bits: u32,
    pub rate: u32,
}

impl Format {
    fn block(&self) -> u32 {
        (self.channels * self.bits / 8).max(1)
    }
}

/// What `IDirectSound::GetCaps` says (as Wine's DirectSound does: no hardware buffers):
/// the dwords at +4 .. +0x10 of `DSCAPS`, the rest 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceCaps {
    pub flags: u32,
    pub min_rate: u32,
    pub max_rate: u32,
    pub primary_buffers: u32,
}

/// A buffer's bytes, shared by a buffer and its duplicates; written while locked through
/// `with_parts` (not by address).
type SoundData = Mutex<Vec<u8>>;

/// A buffer's bytes, a poisoned lock taken as it is.
fn bytes_of(data: &SoundData) -> std::sync::MutexGuard<'_, Vec<u8>> {
    data.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct Buffer {
    data: Arc<SoundData>,
    bytes: u32,
    flags: u32,
    format: Format,
    /// 0: the format's.
    frequency: u32,
    /// In frames, with the fraction the stepping leaves.
    position: f64,
    playing: bool,
    looping: bool,
    volume: i32,
    pan: i32,
    /// Its place in 3D, once its 3D interface was asked for.
    space: Option<Space>,
}

/// A buffer's 3D settings (DirectSound 3D's defaults at first; the game sets only the
/// place and the least distance).
#[derive(Clone, Copy)]
struct Space {
    place: [f32; 3],
    least: f32,
    greatest: f32,
    /// 0 normal, 1 around the listener, 2 not placed.
    mode: u32,
}

impl Space {
    const DEFAULT: Self = Self {
        place: [0.0; 3],
        least: 1.0,
        greatest: 1.0e9,
        mode: 0,
    };

    /// The gains for the left and right channels.
    fn gains(&self, rolloff: f32) -> Option<(f32, f32)> {
        if self.mode == 2 {
            return None;
        }
        let [x, y, z] = self.place;
        let distance = (x * x + y * y + z * z).sqrt();
        if !distance.is_finite() {
            return Some((0.0, 0.0));
        }
        let least = self.least.max(1.0e-6);
        let held = distance.min(self.greatest.max(least));
        let level = if held <= least {
            1.0
        } else {
            portable::powf(least / held, rolloff.max(0.0))
        };
        // -1 all left .. 1 all right, by the direction's x.
        let side = if distance > 1.0e-6 {
            (x / distance).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let angle = (side + 1.0) * std::f32::consts::FRAC_PI_4;
        let both = std::f32::consts::SQRT_2;
        Some((
            level * portable::cosf(angle) * both,
            level * portable::sinf(angle) * both,
        ))
    }
}

enum Kind {
    Device,
    Buffer(Buffer),
    /// A buffer's 3D interface: the buffer's id.
    Space(u32),
    /// The listener (its roll-off is the state's).
    Listener,
}

struct Entry {
    refs: u32,
    kind: Kind,
}

/// The objects by id (what the game holds of each: never 0), in id order: the mix adds the playing
/// buffers in that order, so the same sounds mix to the same samples every run.
struct State {
    entries: BTreeMap<u32, Entry>,
    next: u32,
    primary: Option<u32>,
    /// The listener's roll-off factor (DirectSound's default; the game leaves it).
    rolloff: f32,
    /// The window's focus flag (1: in front), set when the device is created.
    window_focused: Option<&'static crate::session::Dword>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    // Taken even after a panic held it: the audio callback must not panic on it again.
    let mut s = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(s.get_or_insert_with(|| State {
        entries: BTreeMap::new(),
        next: 1,
        primary: None,
        rolloff: 1.0,
        window_focused: None,
    }))
}

fn new_object(s: &mut State, kind: Kind) -> u32 {
    let id = s.next;
    s.next += 1;
    s.entries.insert(id, Entry { refs: 1, kind });
    id
}

fn buffer(s: &mut State, id: u32) -> Option<&mut Buffer> {
    match &mut s.entries.get_mut(&id)?.kind {
        Kind::Buffer(b) => Some(b),
        _ => None,
    }
}

/// The buffer `id` changed by `f`; `E_INVALIDARG` for no such buffer.
fn with_buffer(id: u32, f: impl FnOnce(&mut Buffer)) -> i32 {
    with(|s| match buffer(s, id) {
        Some(b) => {
            f(b);
            OK
        }
        None => E_INVALIDARG,
    })
}

/// The 3D settings of the buffer whose 3D interface is `id`, changed by `f`.
fn with_space(id: u32, f: impl FnOnce(&mut Space)) -> i32 {
    with(|s| {
        let Some(Entry {
            kind: Kind::Space(owner),
            ..
        }) = s.entries.get(&id)
        else {
            return E_INVALIDARG;
        };
        let owner = *owner;
        match buffer(s, owner).and_then(|b| b.space.as_mut()) {
            Some(p) => {
                f(p);
                OK
            }
            None => E_INVALIDARG,
        }
    })
}

/// A primary buffer's size for a format: a quarter of a second, whole frames.
fn primary_bytes(f: &Format) -> u32 {
    (f.rate * f.block() / 4) / f.block() * f.block()
}

// ---- every object (IUnknown) ----

/// `Release`: the references left; at none the object goes (a 3D interface lets its
/// buffer go too).
pub fn release(id: u32) -> u32 {
    with(|s| {
        let Some(e) = s.entries.get_mut(&id) else {
            return 0;
        };
        e.refs -= 1;
        let left = e.refs;
        if left == 0 {
            let owner = match e.kind {
                Kind::Space(owner) => Some(owner),
                _ => None,
            };
            s.entries.remove(&id);
            if s.primary == Some(id) {
                s.primary = None;
            }
            if let Some(owner) = owner
                && let Some(b) = s.entries.get_mut(&owner)
            {
                b.refs -= 1;
                if b.refs == 0 {
                    s.entries.remove(&owner);
                }
            }
        }
        left
    })
}

/// `QueryInterface` by the interface id's first dword: a secondary buffer's 3D interface,
/// the primary buffer's listener; anything else (the property sets) `E_NOINTERFACE`.
pub fn query_interface(id: u32, iid: u32) -> Result<u32, i32> {
    with(|s| {
        let primary = match buffer(s, id) {
            Some(b) => b.flags & PRIMARY != 0,
            None => return Err(E_NOINTERFACE),
        };
        let kind = match (iid, primary) {
            (IID_3D_BUFFER, false) => {
                if let Some(b) = buffer(s, id) {
                    b.space.get_or_insert(Space::DEFAULT);
                }
                // The buffer is kept while its 3D interface is.
                if let Some(e) = s.entries.get_mut(&id) {
                    e.refs += 1;
                }
                Kind::Space(id)
            }
            (IID_3D_LISTENER, true) => Kind::Listener,
            _ => return Err(E_NOINTERFACE),
        };
        Ok(new_object(s, kind))
    })
}

// ---- the device (IDirectSound) ----

/// `CreateSoundBuffer`: a primary buffer (`DSBCAPS_PRIMARYBUFFER` in `flags`) a quarter
/// second of its format; a secondary one `bytes` long. No format: 8-bit stereo at 22050.
pub fn create_sound_buffer(flags: u32, bytes: u32, format: Option<Format>) -> u32 {
    let primary = flags & PRIMARY != 0;
    let format = format.unwrap_or(Format {
        channels: 2,
        bits: 8,
        rate: 22050,
    });
    let bytes = if primary {
        primary_bytes(&format)
    } else {
        bytes
    };
    // A primary buffer's memory is the largest it can be, its formats changing.
    let room = if primary { 48000 * 4 } else { bytes as usize };
    with(|s| {
        let id = new_object(
            s,
            Kind::Buffer(Buffer {
                data: Arc::new(Mutex::new(vec![
                    if format.bits == 8 { 0x80 } else { 0 };
                    room.max(4)
                ])),
                bytes,
                flags,
                format,
                frequency: 0,
                position: 0.0,
                playing: false,
                looping: false,
                volume: 0,
                pan: 0,
                space: None,
            }),
        );
        if primary {
            s.primary = Some(id);
        }
        id
    })
}

/// `GetCaps`.
pub fn device_caps() -> DeviceCaps {
    DeviceCaps {
        flags: CAPS_FLAGS,
        min_rate: 100,
        max_rate: 200_000,
        primary_buffers: 1,
    }
}

/// `DuplicateSoundBuffer`: a buffer sharing the source's data, stopped at its start.
pub fn duplicate_sound_buffer(source: u32) -> Result<u32, i32> {
    with(|s| {
        let Some(b) = buffer(s, source) else {
            return Err(E_INVALIDARG);
        };
        let copy = Buffer {
            data: Arc::clone(&b.data),
            position: 0.0,
            playing: false,
            looping: false,
            ..*b
        };
        Ok(new_object(s, Kind::Buffer(copy)))
    })
}

/// `SetCooperativeLevel`: nothing to do (SDL's stream is shared).
pub fn set_cooperative_level(_device: u32, _window: u32, _level: u32) -> i32 {
    OK
}

// ---- a buffer (IDirectSoundBuffer) ----

/// `GetCaps`: the buffer's flags and length.
pub fn buffer_caps(id: u32) -> Result<(u32, u32), i32> {
    with(|s| {
        buffer(s, id)
            .map(|b| (b.flags, b.bytes))
            .ok_or(E_INVALIDARG)
    })
}

/// `GetCurrentPosition`: the play and write cursors (both where it plays).
pub fn current_position(id: u32) -> Result<(u32, u32), i32> {
    with(|s| {
        let b = buffer(s, id).ok_or(E_INVALIDARG)?;
        let at = (b.position as u32 * b.format.block()) % b.bytes.max(1);
        Ok((at, at))
    })
}

/// `GetVolume`.
pub fn volume(id: u32) -> Result<i32, i32> {
    with(|s| buffer(s, id).map(|b| b.volume).ok_or(E_INVALIDARG))
}

/// `GetStatus`: `DSBSTATUS_PLAYING`, with `_LOOPING`.
pub fn status(id: u32) -> Result<u32, i32> {
    with(|s| {
        let b = buffer(s, id).ok_or(E_INVALIDARG)?;
        Ok(if b.playing {
            STATUS_PLAYING | if b.looping { STATUS_LOOPING } else { 0 }
        } else {
            0
        })
    })
}

/// `Lock`: the two parts (place in the buffer + 1, 0 for none; bytes) of `bytes` from
/// `offset` round the buffer, written through `with_parts`
/// (flag 1: from the play cursor; 2: the whole buffer).
pub fn lock(id: u32, offset: u32, bytes: u32, flags: u32) -> Result<[u32; 4], i32> {
    with(|s| {
        let b = buffer(s, id).ok_or(E_INVALIDARG)?;
        let size = b.bytes.max(1);
        let bytes = if flags & 2 != 0 {
            size
        } else {
            bytes.min(size)
        };
        let offset = if flags & 1 != 0 {
            (b.position as u32 * b.format.block()) % size
        } else {
            offset % size
        };
        let first = bytes.min(size - offset);
        Ok([
            offset + 1,
            first,
            if bytes > first { 1 } else { 0 },
            bytes - first,
        ])
    })
}

/// `f` on the bytes of the parts `lock` gave (`parts`: the first's place + 1 and bytes, the
/// second's; a place 0 is no part, given as empty). `E_INVALIDARG` for no such buffer or
/// parts that are not the buffer's.
pub fn with_parts<R>(
    id: u32,
    parts: [u32; 4],
    f: impl FnOnce(&mut [u8], &mut [u8]) -> R,
) -> Result<R, i32> {
    with(|s| {
        let b = buffer(s, id).ok_or(E_INVALIDARG)?;
        let mut data = bytes_of(&b.data);
        let [p1, n1, p2, n2] = parts;
        let first_at = p1.checked_sub(1).ok_or(E_INVALIDARG)? as usize;
        // The second part is the buffer's start, before the first.
        let (head, tail) = data.split_at_mut_checked(first_at).ok_or(E_INVALIDARG)?;
        let first = tail.get_mut(..n1 as usize).ok_or(E_INVALIDARG)?;
        let second = match p2 {
            0 => &mut [][..],
            _ => head
                .get_mut(p2 as usize - 1..(p2 - 1 + n2) as usize)
                .ok_or(E_INVALIDARG)?,
        };
        Ok(f(first, second))
    })
}

/// `Unlock`: the bytes are the buffer's already.
pub fn unlock(_id: u32, _parts: [u32; 4]) -> i32 {
    OK
}

/// `Play` (`DSBPLAY_LOOPING` in `flags`; a primary buffer always loops).
pub fn play(id: u32, flags: u32) -> i32 {
    with_buffer(id, |b| {
        b.playing = true;
        b.looping = flags & LOOPING != 0 || b.flags & PRIMARY != 0;
    })
}

/// `SetCurrentPosition`.
pub fn set_current_position(id: u32, at: u32) -> i32 {
    with_buffer(id, |b| b.position = (at / b.format.block()) as f64)
}

/// `SetFormat` (the primary buffer only): its bytes silence in the new format.
pub fn set_format(id: u32, f: Format) -> i32 {
    if f.rate == 0 || f.rate > 48000 || !matches!(f.bits, 8 | 16) || !matches!(f.channels, 1 | 2) {
        return E_INVALIDARG;
    }
    with(|s| match buffer(s, id) {
        Some(b) if b.flags & PRIMARY != 0 => {
            // Silence in the new format: the 8-bit one's 0x80 bytes read as 16-bit are
            // -32640, a jump to near full scale (clicks at the opening).
            let silence = if f.bits == 8 { 0x80 } else { 0 };
            bytes_of(&b.data).fill(silence);
            b.format = f;
            b.bytes = primary_bytes(&f);
            b.position = 0.0;
            crate::framework::log(&format!(
                "platform_sound: the primary buffer's format: {} Hz, {} bits, {} channels",
                f.rate, f.bits, f.channels
            ));
            OK
        }
        _ => E_INVALIDARG,
    })
}

/// `SetVolume` (hundredths of a decibel, 0 the loudest).
pub fn set_volume(id: u32, volume: i32) -> i32 {
    with_buffer(id, |b| b.volume = volume)
}

/// `SetPan`.
pub fn set_pan(id: u32, pan: i32) -> i32 {
    with_buffer(id, |b| b.pan = pan)
}

/// `SetFrequency` (0: the format's).
pub fn set_frequency(id: u32, frequency: u32) -> i32 {
    with_buffer(id, |b| b.frequency = frequency)
}

/// `Stop`.
pub fn stop(id: u32) -> i32 {
    with_buffer(id, |b| b.playing = false)
}

/// `Restore`: a buffer here is never lost.
pub fn restore(_id: u32) -> i32 {
    OK
}

// ---- 3D (IDirectSound3DBuffer, IDirectSound3DListener) ----

/// `SetPosition(x, y, z, apply)`: deferred or not, taken at once (the game commits each turn).
pub fn set_place(id: u32, place: [f32; 3]) -> i32 {
    with_space(id, |p| p.place = place)
}

/// `SetMinDistance(value, apply)`, taken at once.
pub fn set_min_distance(id: u32, value: f32) -> i32 {
    with_space(id, |p| p.least = value)
}

/// The listener's `CommitDeferredSettings`: everything is taken at once.
pub fn commit_deferred(_listener: u32) -> i32 {
    OK
}

// ---- the mixer ----

/// Hundredths of a decibel to a gain.
fn gain(centibels: i32) -> f32 {
    portable::powf(10.0, centibels.clamp(-10000, 0) as f32 / 2000.0)
}

/// The secondary buffers' gain: a quarter (-12 dB), the level the game's own mixer gives
/// its channels at full volume (`volume * volume_factor` at most 0x1000, `>> 14`).
const SECONDARY_GAIN: f32 = 0.25;

/// One buffer added into `out` (stereo frames at `OUT_RATE`), its position moved on.
fn mix_into(b: &mut Buffer, master: f32, rolloff: f32, out: &mut [f32]) {
    let f = b.format;
    let block = f.block() as usize;
    let frames = (b.bytes as usize / block).max(1);
    let rate = if b.frequency != 0 {
        b.frequency
    } else {
        f.rate
    };
    let step = rate as f64 / OUT_RATE as f64;
    let g = gain(b.volume) * master;
    let (left, right) = match b.space.and_then(|p| p.gains(rolloff)) {
        Some((l, r)) => (g * l, g * r),
        None if b.pan < 0 => (g, g * gain(b.pan)),
        None => (g * gain(-b.pan), g),
    };
    let data = bytes_of(&b.data);
    let data: &[u8] = &data;
    let sample = |at: usize| -> f32 {
        if f.bits == 8 {
            (data[at] as f32 - 128.0) / 128.0
        } else {
            i16::from_le_bytes([data[at], data[at + 1]]) as f32 / 32768.0
        }
    };
    let bytes = (f.bits / 8) as usize;
    for frame in out.as_chunks_mut::<2>().0 {
        let mut at = b.position as usize;
        if at >= frames {
            if !b.looping {
                b.playing = false;
                b.position = 0.0;
                return;
            }
            b.position -= frames as f64 * (b.position / frames as f64).floor();
            at = b.position as usize;
        }
        // Linear between this frame and the next (the first again when looping, this
        // one at the end), as DirectSound's resampling.
        let next = if at + 1 < frames {
            at + 1
        } else if b.looping {
            0
        } else {
            at
        };
        let t = (b.position - at as f64) as f32;
        let both = |n: usize| {
            let first = n * block;
            let l = sample(first);
            let r = if f.channels == 2 {
                sample(first + bytes)
            } else {
                l
            };
            (l, r)
        };
        let ((l0, r0), (l1, r1)) = (both(at), both(next));
        frame[0] += (l0 + (l1 - l0) * t) * left;
        frame[1] += (r0 + (r1 - r0) * t) * right;
        b.position += step;
    }
}

/// `DSBCAPS_STICKYFOCUS` and `DSBCAPS_GLOBALFOCUS`: heard without the window's focus.
const STICKY_FOCUS: u32 = 0x4000;
const GLOBAL_FOCUS: u32 = 0x8000;

/// The loudest sample sent to the device since the remote control last asked.
#[cfg(feature = "remote")]
pub static LAST_PEAK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Whether the audio callback has panicked: it gives silence from then on.
static FEED_PANICKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// SDL's audio callback: the playing buffers mixed into what the stream wants (none when it
/// wants less than a frame).
fn feed(wanted: i32) -> Option<Vec<i16>> {
    use std::sync::atomic::Ordering::Relaxed;
    let frames = (wanted.max(0) / 4) as usize;
    if frames == 0 {
        return None;
    }
    // The mix-hash check mixes on the audio thread instead (`audio::mix_hash`).
    #[cfg(feature = "remote")]
    if crate::audio::mix_hash::deterministic() {
        return Some(vec![0i16; frames * 2]);
    }
    // A panic must not unwind into SDL (it would end the program, unreported).
    let mixed = if FEED_PANICKED.load(Relaxed) {
        None
    } else {
        std::panic::catch_unwind(|| mixed_frames(frames)).ok()
    };
    Some(mixed.unwrap_or_else(|| {
        if !FEED_PANICKED.swap(true, Relaxed) {
            crate::framework::log(
                "platform_sound: the audio callback panicked: silence from now on",
            );
        }
        vec![0i16; frames * 2]
    }))
}

/// The playing buffers mixed into `frames` stereo frames for the device.
pub(crate) fn mixed_frames(frames: usize) -> Vec<i16> {
    let mut out = vec![0f32; frames * 2];
    // DirectSound's focus: while the window is not in front, a buffer made without
    // `DSBCAPS_GLOBALFOCUS` or `DSBCAPS_STICKYFOCUS` plays on unheard (the game asks for
    // neither, so it goes quiet and comes back with the focus).
    let mut unheard = Vec::new();
    with(|s| {
        // Before the device exists nothing plays; the flag is then taken as set.
        let focused = s.window_focused.is_none_or(|flag| flag.get() == 1);
        let master = s
            .primary
            .and_then(|p| match &s.entries.get(&p)?.kind {
                Kind::Buffer(b) => Some(gain(b.volume)),
                _ => None,
            })
            .unwrap_or(1.0);
        let rolloff = s.rolloff;
        for e in s.entries.values_mut() {
            if let Kind::Buffer(b) = &mut e.kind
                && b.playing
            {
                // The primary buffer's own volume is the master's; its data is not
                // scaled twice (nor given the secondary buffers' headroom, which the game's
                // mixer has written into it already).
                let m = if b.flags & PRIMARY != 0 {
                    1.0
                } else {
                    master * SECONDARY_GAIN
                };
                if focused || b.flags & (GLOBAL_FOCUS | STICKY_FOCUS) != 0 {
                    mix_into(b, m, rolloff, &mut out);
                } else {
                    unheard.clear();
                    unheard.resize(out.len(), 0f32);
                    mix_into(b, m, rolloff, &mut unheard);
                }
            }
        }
    });
    let pcm: Vec<i16> = out
        .iter()
        .map(|x| (x * 32767.0).clamp(-32768.0, 32767.0) as i16)
        .collect();
    #[cfg(feature = "remote")]
    if let Some(peak) = pcm.iter().map(|sample| sample.unsigned_abs() as u32).max() {
        LAST_PEAK.fetch_max(peak, std::sync::atomic::Ordering::Relaxed);
    }
    pcm
}

static OPENED: Mutex<bool> = Mutex::new(false);

/// `DirectSoundCreate` for the standalone: the device, SDL's audio opened the first time.
/// Fails (as `DirectSoundCreate` can, with `DSERR_NODRIVER`) when SDL cannot.
pub fn create_device(window_focused: &'static crate::session::Dword) -> Result<u32, i32> {
    with(|s| s.window_focused = Some(window_focused));
    let mut opened = OPENED.lock().unwrap();
    if !*opened {
        match crate::ffi::sdl::start_audio_output(OUT_RATE, feed) {
            Ok(()) => {}
            Err(crate::ffi::sdl::AudioError::Init) => {
                crate::framework::log("platform_sound: SDL audio cannot be started");
                return Err(NO_DRIVER);
            }
            Err(crate::ffi::sdl::AudioError::NoDevice) => {
                crate::framework::log("platform_sound: no SDL audio device");
                return Err(NO_DRIVER);
            }
        }
        *opened = true;
    }
    Ok(with(|s| new_object(s, Kind::Device)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objects_lock_round_and_go_with_their_references() {
        let format = Format {
            channels: 2,
            bits: 16,
            rate: 22050,
        };
        let b = create_sound_buffer(0x80, 400, Some(format));
        assert_ne!(b, 0);
        assert_eq!(buffer_caps(b), Ok((0x80, 400)));
        // The 3D interface of a secondary buffer, and none of the property sets.
        let space = query_interface(b, IID_3D_BUFFER).unwrap();
        assert_eq!(query_interface(b, IID_3D_LISTENER), Err(E_NOINTERFACE));
        assert_eq!(query_interface(b, 0x31EF_AC30), Err(E_NOINTERFACE));
        assert_eq!(set_place(space, [1.0, 2.0, 3.0]), OK);
        assert_eq!(set_place(b, [1.0, 2.0, 3.0]), E_INVALIDARG);
        // A lock past the end goes round to the start.
        assert_eq!(set_current_position(b, 0), OK);
        let [p1, n1, p2, n2] = lock(b, 300, 200, 0).unwrap();
        assert_eq!((n1, n2), (100, 100));
        assert_eq!(p1 - p2, 300);
        assert_eq!(play(b, LOOPING), OK);
        assert_eq!(status(b), Ok(STATUS_PLAYING | STATUS_LOOPING));
        assert_eq!(stop(b), OK);
        assert_eq!(status(b), Ok(0));
        // A duplicate shares the data; the buffer stays while its 3D interface does.
        let copy = duplicate_sound_buffer(b).unwrap();
        assert_eq!(lock(copy, 300, 1, 0).unwrap()[0], p1);
        assert_eq!(release(b), 1);
        assert_eq!(status(b), Ok(0));
        assert_eq!(release(space), 0);
        assert_eq!(status(b), Err(E_INVALIDARG));
        assert_eq!(release(copy), 0);
    }

    const FROZEN_FRAMES: usize = 119262;
    const FROZEN_HASH: u64 = 10_094_763_369_305_457_242;

    /// Serialises the tests that play buffers: the mixer pulls every playing buffer there is.
    static PLAYING: Mutex<()> = Mutex::new(());

    /// FNV-1a over a stream's bytes.
    fn fnv(hash: u64, bytes: &[u8]) -> u64 {
        bytes.iter().fold(hash, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    }

    /// A buffer of `bytes` filled with a fixed pseudo-random pattern (`seed`), made as the game
    /// does: created, locked, written, unlocked.
    fn filled_buffer(flags: u32, bytes: u32, format: Format, seed: u32) -> u32 {
        let id = create_sound_buffer(flags, bytes, Some(format));
        let parts = lock(id, 0, bytes, 2).unwrap();
        with_parts(id, parts, |first, _| {
            let mut x = seed;
            for byte in first.iter_mut() {
                x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                *byte = (x >> 24) as u8;
            }
        })
        .unwrap();
        assert_eq!(unlock(id, parts), OK);
        id
    }

    /// Pulls the device mix in an odd schedule from six buffers (looped, resampled, panned, placed
    /// in 3D, one ending part-way through a pull, one stopped, the primary one with its volume
    /// as the master), of which `only` (or all) play. Returns the samples of each pull.
    fn pulled(only: Option<usize>) -> Vec<Vec<i16>> {
        let stereo16 = Format {
            channels: 2,
            bits: 16,
            rate: 22050,
        };
        let mono8 = Format {
            channels: 1,
            bits: 8,
            rate: 11025,
        };
        let mono16 = Format {
            channels: 1,
            bits: 16,
            rate: 8000,
        };
        let looped = filled_buffer(GLOBAL_FOCUS, 4000, stereo16, 1);
        assert_eq!(set_volume(looped, -600), OK);
        let resampled = filled_buffer(GLOBAL_FOCUS, 300, mono8, 2);
        assert_eq!(set_frequency(resampled, 33075), OK);
        assert_eq!(set_pan(resampled, -1500), OK);
        assert_eq!(set_volume(resampled, -200), OK);
        let placed = filled_buffer(GLOBAL_FOCUS, 1600, mono16, 3);
        let space = query_interface(placed, IID_3D_BUFFER).unwrap();
        assert_eq!(set_place(space, [3.0, 0.0, 4.0]), OK);
        assert_eq!(set_min_distance(space, 2.0), OK);
        let panned = filled_buffer(GLOBAL_FOCUS, 2000, stereo16, 4);
        assert_eq!(set_pan(panned, 2500), OK);
        assert_eq!(set_current_position(panned, 400), OK);
        let stopped = filled_buffer(GLOBAL_FOCUS, 800, stereo16, 5);
        let primary = filled_buffer(PRIMARY | GLOBAL_FOCUS, 0, stereo16, 6);
        assert_eq!(set_volume(primary, -300), OK);
        let all = [looped, resampled, placed, panned, stopped, primary];
        for (n, id) in all.into_iter().enumerate() {
            // The stopped one is played and stopped again; the others play looping or once.
            let flags = if matches!(n, 0 | 2 | 4) { LOOPING } else { 0 };
            assert_eq!(play(id, flags), OK);
            if n == 4 || only.is_some_and(|only| only != n) {
                assert_eq!(stop(id), OK);
            }
        }
        let out = [1usize, 7, 64, 1023, 333, 4096, 5, 2000, 12345, 3]
            .map(mixed_frames)
            .to_vec();
        for id in all {
            release(id);
        }
        release(space);
        out
    }

    /// Guards the sound the player hears. Each of six buffers (looped, resampled, panned, placed
    /// in 3D, ending part-way through a pull, stopped, the primary one as master volume) played
    /// alone and pulled in an odd schedule gives the frozen samples; all six together give the
    /// sum of the six, to within the rounding of the order they are added in (the buffers are
    /// visited in a hash map's order, which changes from run to run, so only the single-buffer
    /// samples are frozen exactly).
    #[test]
    fn the_device_mix_of_playing_buffers_is_frozen() {
        let _held = PLAYING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Other tests may leave buffers playing in the one global state: they are paused for
        // the length of this test and set going again after it.
        let foreign: Vec<u32> = with(|s| {
            let mut paused = Vec::new();
            for (id, e) in &mut s.entries {
                if let Kind::Buffer(b) = &mut e.kind
                    && b.playing
                {
                    b.playing = false;
                    paused.push(*id);
                }
            }
            paused
        });
        let solo: Vec<Vec<Vec<i16>>> = (0..6).map(|n| pulled(Some(n))).collect();
        let together = pulled(None);
        for id in foreign {
            with_buffer(id, |b| b.playing = true);
        }
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        let mut frames = 0;
        for run in &solo {
            for pull in run {
                frames += pull.len() / 2;
                let bytes: Vec<u8> = pull.iter().flat_map(|s| s.to_le_bytes()).collect();
                hash = fnv(hash, &bytes);
            }
        }
        assert_eq!((frames, hash), (FROZEN_FRAMES, FROZEN_HASH));
        // The mix of all of them is what the six give alone (every sample off a clip's edge).
        let mut compared = 0;
        for (n, pull) in together.iter().enumerate() {
            for (at, &mixed) in pull.iter().enumerate() {
                let sum: i32 = solo.iter().map(|run| i32::from(run[n][at])).sum();
                if sum.abs() < 30000 {
                    assert!((i32::from(mixed) - sum).abs() <= 6, "pull {n} sample {at}");
                    compared += 1;
                }
            }
        }
        assert!(compared > 30000, "{compared} samples compared");
    }

    /// A buffer made without a focus flag is heard only while the window's flag is 1; one made
    /// with `GLOBAL_FOCUS` is heard either way.
    #[test]
    fn a_buffer_without_the_focus_flags_is_silent_while_the_window_is_not_in_front() {
        static FOCUS: crate::session::Dword = crate::session::Dword::new(0);
        let _held = PLAYING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let format = Format {
            channels: 2,
            bits: 16,
            rate: 22050,
        };
        let foreign: Vec<u32> = with(|s| {
            s.window_focused = Some(&FOCUS);
            let mut paused = Vec::new();
            for (id, e) in &mut s.entries {
                if let Kind::Buffer(b) = &mut e.kind
                    && b.playing
                {
                    b.playing = false;
                    paused.push(*id);
                }
            }
            paused
        });
        let heard = |flags: u32, focus: u32| {
            FOCUS.set(focus);
            let id = filled_buffer(flags, 4000, format, 7);
            assert_eq!(play(id, LOOPING), OK);
            let pcm = mixed_frames(64);
            assert_eq!(stop(id), OK);
            release(id);
            pcm.iter().any(|&sample| sample != 0)
        };
        assert!(!heard(0, 0), "no flag, window behind");
        assert!(heard(0, 1), "no flag, window in front");
        assert!(heard(GLOBAL_FOCUS, 0), "global focus, window behind");
        for id in foreign {
            with_buffer(id, |b| b.playing = true);
        }
    }
}
