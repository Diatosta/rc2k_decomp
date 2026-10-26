//! DirectSound for the standalone program, on one SDL3 audio stream (the owner chose it,
//! 2026-10-15: "Emulate DirectSound").
//!
//! The ported audio code calls DirectSound only through COM method tables. Here those are
//! Rust functions: a device, its buffers (the primary one and the secondary ones, with
//! duplicates sharing their data), the buffers' 3D interfaces and the 3D listener. A small
//! software mixer in SDL's audio callback plays every playing buffer: its data (8 or 16
//! bits, mono or stereo) at its frequency, with its volume and pan, into 44100 Hz 16-bit
//! stereo; the primary buffer too, which in the Dolby mode is where the game's own mixer
//! writes, so the music (a secondary buffer) is heard there as well (section 16's Dolby
//! bug does not happen here). The primary buffer's volume is the whole output's.
//! 3D placing (2026-10-19; before, the 3D calls did nothing and every placed sound played
//! as near): a buffer's 3D interface keeps its place, its least and greatest distances and
//! its mode; the listener stays where the game leaves it (at the origin, facing +z: the
//! game places the sounds around the listener itself) with its roll-off factor. A placed
//! buffer is heard at DirectSound 3D's distance (the least distance over the distance,
//! to the roll-off's power, held at the greatest distance) and panned by its direction
//! (equal power); its own pan is not used then, as in DirectSound.
//! Not done: Doppler, cones, EAX (its set-up is unported game code, which the standalone
//! passes by).
//! Every method has the arity of DirectSound's: these are `stdcall`, the callee takes the
//! arguments off the stack.

use sdl3_sys::everything::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const OUT_RATE: u32 = 44100;
const OK: i32 = 0;
const E_NOINTERFACE: i32 = 0x8000_4002u32 as i32;
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

/// A method table: the methods, shared by every thread, and (made at the first use) the
/// table the game reads, of their game addresses.
struct Table<const N: usize>([*const (); N], std::sync::OnceLock<[u32; N]>);
unsafe impl<const N: usize> Sync for Table<N> {}

impl<const N: usize> Table<N> {
    /// The game's table: its game address.
    fn start(&'static self) -> u32 {
        let game = self
            .1
            .get_or_init(|| self.0.map(crate::pointers::game_address));
        crate::pointers::game_address(game.as_ptr())
    }
}

/// What the game holds: its method table first, as for any COM object.
#[repr(C)]
struct Object {
    /// The table's game address.
    table: u32,
    id: u32,
}

#[derive(Clone, Copy)]
struct Format {
    channels: u32,
    bits: u32,
    rate: u32,
}

impl Format {
    fn block(&self) -> u32 {
        (self.channels * self.bits / 8).max(1)
    }
}

/// A buffer's bytes, a region while they live (the game's code writes them by address once
/// locked: `pointers`); shared by a buffer and its duplicates.
struct SoundData {
    bytes: Vec<u8>,
    _lent: crate::pointers::Lent,
}

impl SoundData {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            _lent: crate::pointers::Lent::new(&bytes),
            bytes,
        }
    }
}

impl std::ops::Deref for SoundData {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.bytes
    }
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

/// A buffer's 3D settings (DirectSound 3D's defaults at first).
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
            (least / held).powf(rolloff.max(0.0))
        };
        // -1 all left .. 1 all right, by the direction's x.
        let side = if distance > 1.0e-6 {
            (x / distance).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let angle = (side + 1.0) * std::f32::consts::FRAC_PI_4;
        let both = std::f32::consts::SQRT_2;
        Some((level * angle.cos() * both, level * angle.sin() * both))
    }
}

enum Kind {
    Device,
    Buffer(Buffer),
    /// A buffer's 3D interface: the buffer's id.
    Space(u32),
    /// The listener (its roll-off is the state's).
    Quiet,
}

struct Entry {
    object: Box<Object>,
    refs: u32,
    kind: Kind,
    /// The object a region while it lives (the game's code reads its table by address:
    /// `audio::com`).
    _lent: crate::pointers::Lent,
}

struct State {
    entries: HashMap<u32, Entry>,
    next: u32,
    primary: Option<u32>,
    /// The listener's roll-off factor.
    rolloff: f32,
}

// The raw pointers in it are the game's to use; every access goes through the mutex.
unsafe impl Send for State {}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut s = STATE.lock().unwrap();
    f(s.get_or_insert_with(|| State {
        entries: HashMap::new(),
        next: 1,
        primary: None,
        rolloff: 1.0,
    }))
}

fn id_of(this: u32) -> u32 {
    get(this + std::mem::offset_of!(Object, id) as u32)
}

/// A dword written where the game asked (none for a null address); an address outside the
/// game's memory ends the program.
fn put(at: u32, value: u32) {
    put_as(at, value)
}

fn put_as<T: Copy>(at: u32, value: T) {
    if at != 0 {
        crate::pointers::write_or_end(at, value, "a DirectSound method's result")
    }
}

fn get(at: u32) -> u32 {
    crate::pointers::read_or_end(at, "a DirectSound method's argument")
}

fn new_object(s: &mut State, table: u32, kind: Kind) -> u32 {
    let id = s.next;
    s.next += 1;
    let object = Box::new(Object { table, id });
    // SAFETY: the object's own bytes, for the registry only.
    let bytes = unsafe {
        std::slice::from_raw_parts(&*object as *const Object as *const u8, size_of::<Object>())
    };
    let lent = crate::pointers::Lent::new(bytes);
    let at = lent.addr();
    s.entries.insert(
        id,
        Entry {
            _lent: lent,
            object,
            refs: 1,
            kind,
        },
    );
    at
}

fn buffer(s: &mut State, this: u32) -> Option<&mut Buffer> {
    match &mut s.entries.get_mut(&id_of(this))?.kind {
        Kind::Buffer(b) => Some(b),
        _ => None,
    }
}

/// The 3D settings of the buffer whose 3D interface is `this`.
fn space(s: &mut State, this: u32) -> Option<&mut Space> {
    let id = match s.entries.get(&id_of(this))?.kind {
        Kind::Space(id) => id,
        _ => return None,
    };
    match &mut s.entries.get_mut(&id)?.kind {
        Kind::Buffer(b) => b.space.as_mut(),
        _ => None,
    }
}

fn read_format(at: u32) -> Format {
    let word = |o: u32| crate::pointers::read_or_end::<u16>(at + o, "a wave format") as u32;
    Format {
        channels: word(2),
        rate: get(at + 4),
        bits: word(0xE),
    }
}

/// A primary buffer's size for a format: a quarter of a second, whole frames.
fn primary_bytes(f: &Format) -> u32 {
    (f.rate * f.block() / 4) / f.block() * f.block()
}

// ---- the methods every object has ----

extern "system" fn add_ref(this: u32) -> u32 {
    with(|s| {
        s.entries.get_mut(&id_of(this)).map_or(0, |e| {
            e.refs += 1;
            e.refs
        })
    })
}

extern "system" fn release(this: u32) -> u32 {
    with(|s| {
        let id = id_of(this);
        let Some(e) = s.entries.get_mut(&id) else {
            return 0;
        };
        e.refs -= 1;
        let left = e.refs;
        if left == 0 {
            // The object's memory goes too: the game must not use it again (it does not).
            let _ = &e.object;
            let owner = match e.kind {
                Kind::Space(owner) => Some(owner),
                _ => None,
            };
            s.entries.remove(&id);
            if s.primary == Some(id) {
                s.primary = None;
            }
            // A 3D interface held its buffer.
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

extern "system" fn no_interface(_: u32, _: u32, out: u32) -> i32 {
    put(out, 0);
    E_NOINTERFACE
}

extern "system" fn ok0(_: u32) -> i32 {
    OK
}
extern "system" fn ok1(_: u32, _: u32) -> i32 {
    OK
}
extern "system" fn ok2(_: u32, _: u32, _: u32) -> i32 {
    OK
}
extern "system" fn ok3(_: u32, _: u32, _: u32, _: u32) -> i32 {
    OK
}
extern "system" fn ok4(_: u32, _: u32, _: u32, _: u32, _: u32) -> i32 {
    OK
}
#[allow(clippy::too_many_arguments)] // IDirectSound3DListener::SetOrientation's arity
extern "system" fn ok7(_: u32, _: u32, _: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> i32 {
    OK
}
/// A `Get..` whose answer is not looked at by the game: a 0 written.
extern "system" fn zero1(_: u32, out: u32) -> i32 {
    put(out, 0);
    OK
}
extern "system" fn zero2(_: u32, a: u32, b: u32) -> i32 {
    put(a, 0);
    put(b, 0);
    OK
}

// ---- the device (IDirectSound) ----

extern "system" fn create_buffer(_: u32, desc: u32, out: u32, _outer: u32) -> i32 {
    if desc == 0 || out == 0 {
        return E_INVALIDARG;
    }
    let flags = get(desc + 4);
    let format_at = get(desc + 0x10);
    let primary = flags & PRIMARY != 0;
    let format = if format_at != 0 {
        read_format(format_at)
    } else {
        Format {
            channels: 2,
            bits: 8,
            rate: 22050,
        }
    };
    let bytes = if primary {
        primary_bytes(&format)
    } else {
        get(desc + 8)
    };
    // A primary buffer's memory is the largest it can be, its formats changing.
    let room = if primary { 48000 * 4 } else { bytes as usize };
    let at = with(|s| {
        let at = new_object(
            s,
            BUFFER.start(),
            Kind::Buffer(Buffer {
                data: Arc::new(SoundData::new(vec![
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
            s.primary = Some(id_of(at));
        }
        at
    });
    put(out, at);
    OK
}

extern "system" fn device_caps(_: u32, caps: u32) -> i32 {
    for n in 1..0x18 {
        put(caps + n * 4, 0);
    }
    put(caps + 4, CAPS_FLAGS);
    put(caps + 8, 100);
    put(caps + 0xC, 200_000);
    put(caps + 0x10, 1);
    OK
}

extern "system" fn duplicate(_: u32, source: u32, out: u32) -> i32 {
    with(|s| {
        let Some(b) = buffer(s, source) else {
            return E_INVALIDARG;
        };
        let copy = Buffer {
            data: Arc::clone(&b.data),
            position: 0.0,
            playing: false,
            looping: false,
            ..*b
        };
        let at = new_object(s, BUFFER.start(), Kind::Buffer(copy));
        put(out, at);
        OK
    })
}

static DEVICE: Table<11> = Table(
    [
        no_interface as *const (),
        add_ref as *const (),
        release as *const (),
        create_buffer as *const (),
        device_caps as *const (),
        duplicate as *const (),
        ok2 as *const (),   // SetCooperativeLevel
        ok0 as *const (),   // Compact
        zero1 as *const (), // GetSpeakerConfig
        ok1 as *const (),   // SetSpeakerConfig
        ok1 as *const (),   // Initialize
    ],
    std::sync::OnceLock::new(),
);

// ---- a buffer (IDirectSoundBuffer) ----

extern "system" fn buffer_query(this: u32, iid: u32, out: u32) -> i32 {
    let first = { get(iid) };
    with(|s| {
        let primary = buffer(s, this).is_some_and(|b| b.flags & PRIMARY != 0);
        let table = match (first, primary) {
            (IID_3D_BUFFER, false) => BUFFER_3D.start(),
            (IID_3D_LISTENER, true) => LISTENER.start(),
            _ => {
                put(out, 0);
                return E_NOINTERFACE;
            }
        };
        let kind = if primary {
            Kind::Quiet
        } else {
            let id = id_of(this);
            if let Some(b) = buffer(s, this) {
                b.space.get_or_insert(Space::DEFAULT);
            }
            // The buffer is kept while its 3D interface is.
            if let Some(e) = s.entries.get_mut(&id) {
                e.refs += 1;
            }
            Kind::Space(id)
        };
        let at = new_object(s, table, kind);
        put(out, at);
        OK
    })
}

extern "system" fn buffer_caps(this: u32, caps: u32) -> i32 {
    with(|s| {
        let Some(b) = buffer(s, this) else {
            return E_INVALIDARG;
        };
        put(caps + 4, b.flags);
        put(caps + 8, b.bytes);
        put(caps + 0xC, 0);
        put(caps + 0x10, 0);
        OK
    })
}

extern "system" fn position(this: u32, play: u32, write: u32) -> i32 {
    with(|s| {
        let Some(b) = buffer(s, this) else {
            return E_INVALIDARG;
        };
        let at = (b.position as u32 * b.format.block()) % b.bytes.max(1);
        put(play, at);
        put(write, at);
        OK
    })
}

extern "system" fn get_format(this: u32, format: u32, size: u32, written: u32) -> i32 {
    with(|s| {
        let Some(b) = buffer(s, this) else {
            return E_INVALIDARG;
        };
        let f = b.format;
        if format != 0 && size >= 0x10 {
            put_as(format, 1u16);
            put_as(format + 2, f.channels as u16);
            put(format + 4, f.rate);
            put(format + 8, f.rate * f.block());
            put_as(format + 0xC, f.block() as u16);
            put_as(format + 0xE, f.bits as u16);
        }
        put(written, 0x10);
        OK
    })
}

macro_rules! getter {
    ($name:ident, $field:ident) => {
        unsafe extern "system" fn $name(this: u32, out: u32) -> i32 {
            with(|s| match buffer(s, this) {
                Some(b) => {
                    {
                        put(out, b.$field as u32)
                    };
                    OK
                }
                None => E_INVALIDARG,
            })
        }
    };
}
getter!(get_volume, volume);
getter!(get_pan, pan);
getter!(get_frequency, frequency);

extern "system" fn status(this: u32, out: u32) -> i32 {
    with(|s| match buffer(s, this) {
        Some(b) => {
            let st = if b.playing {
                STATUS_PLAYING | if b.looping { STATUS_LOOPING } else { 0 }
            } else {
                0
            };
            put(out, st);
            OK
        }
        None => E_INVALIDARG,
    })
}

#[allow(clippy::too_many_arguments)] // IDirectSoundBuffer::Lock's arity
extern "system" fn lock(
    this: u32,
    offset: u32,
    bytes: u32,
    p1: u32,
    n1: u32,
    p2: u32,
    n2: u32,
    flags: u32,
) -> i32 {
    with(|s| {
        let Some(b) = buffer(s, this) else {
            return E_INVALIDARG;
        };
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
        let base = crate::pointers::game_address(b.data.as_ptr());
        put(p1, base + offset);
        put(n1, first);
        put(p2, if bytes > first { base } else { 0 });
        put(n2, bytes - first);
        OK
    })
}

extern "system" fn play(this: u32, _: u32, _: u32, flags: u32) -> i32 {
    with(|s| match buffer(s, this) {
        Some(b) => {
            b.playing = true;
            b.looping = flags & LOOPING != 0 || b.flags & PRIMARY != 0;
            OK
        }
        None => E_INVALIDARG,
    })
}

extern "system" fn set_position(this: u32, at: u32) -> i32 {
    with(|s| match buffer(s, this) {
        Some(b) => {
            b.position = (at / b.format.block()) as f64;
            OK
        }
        None => E_INVALIDARG,
    })
}

unsafe extern "system" fn set_format(this: u32, format: u32) -> i32 {
    with(|s| match buffer(s, this) {
        Some(b) if b.flags & PRIMARY != 0 && format != 0 => {
            let f = { read_format(format) };
            if f.rate == 0
                || f.rate > 48000
                || !matches!(f.bits, 8 | 16)
                || !matches!(f.channels, 1 | 2)
            {
                return E_INVALIDARG;
            }
            // Silence in the new format: the 8-bit one's 0x80 bytes read as 16-bit are
            // -32640, a jump to near full scale (the clicks at the opening, 2026-10-15).
            let silence = if f.bits == 8 { 0x80 } else { 0 };
            unsafe { std::ptr::write_bytes(b.data.as_ptr() as *mut u8, silence, b.data.len()) };
            b.format = f;
            b.bytes = primary_bytes(&f);
            b.position = 0.0;
            OK
        }
        _ => E_INVALIDARG,
    })
}

macro_rules! setter {
    ($name:ident, $field:ident, $ty:ty) => {
        unsafe extern "system" fn $name(this: u32, value: u32) -> i32 {
            with(|s| match buffer(s, this) {
                Some(b) => {
                    b.$field = value as $ty;
                    OK
                }
                None => E_INVALIDARG,
            })
        }
    };
}
setter!(set_volume, volume, i32);
setter!(set_pan, pan, i32);
setter!(set_frequency, frequency, u32);

extern "system" fn stop(this: u32) -> i32 {
    with(|s| match buffer(s, this) {
        Some(b) => {
            b.playing = false;
            OK
        }
        None => E_INVALIDARG,
    })
}

static BUFFER: Table<21> = Table(
    [
        buffer_query as *const (),
        add_ref as *const (),
        release as *const (),
        buffer_caps as *const (),
        position as *const (),
        get_format as *const (),
        get_volume as *const (),
        get_pan as *const (),
        get_frequency as *const (),
        status as *const (),
        ok2 as *const (), // Initialize
        lock as *const (),
        play as *const (),
        set_position as *const (),
        set_format as *const (),
        set_volume as *const (),
        set_pan as *const (),
        set_frequency as *const (),
        stop as *const (),
        ok4 as *const (), // Unlock
        ok0 as *const (), // Restore
    ],
    std::sync::OnceLock::new(),
);

// ---- 3D (IDirectSound3DBuffer, IDirectSound3DListener) ----

fn float(bits: u32) -> f32 {
    f32::from_bits(bits)
}

/// `SetPosition(x, y, z, apply)`: deferred or not, taken at once (the game commits each
/// turn).
extern "system" fn set_place(this: u32, x: u32, y: u32, z: u32, _: u32) -> i32 {
    with(|s| match space(s, this) {
        Some(p) => {
            p.place = [float(x), float(y), float(z)];
            OK
        }
        None => E_INVALIDARG,
    })
}

extern "system" fn get_place(this: u32, out: u32) -> i32 {
    with(|s| match space(s, this) {
        Some(p) => {
            for (n, v) in p.place.iter().enumerate() {
                put(out + 4 * n as u32, v.to_bits());
            }
            OK
        }
        None => E_INVALIDARG,
    })
}

extern "system" fn set_least(this: u32, value: u32, _: u32) -> i32 {
    with(|s| match space(s, this) {
        Some(p) => {
            p.least = float(value);
            OK
        }
        None => E_INVALIDARG,
    })
}

extern "system" fn set_greatest(this: u32, value: u32, _: u32) -> i32 {
    with(|s| match space(s, this) {
        Some(p) => {
            p.greatest = float(value);
            OK
        }
        None => E_INVALIDARG,
    })
}

extern "system" fn set_mode(this: u32, value: u32, _: u32) -> i32 {
    with(|s| match space(s, this) {
        Some(p) => {
            p.mode = value;
            OK
        }
        None => E_INVALIDARG,
    })
}

macro_rules! get_space {
    ($name:ident, $field:ident, $to:expr) => {
        unsafe extern "system" fn $name(this: u32, out: u32) -> i32 {
            with(|s| match space(s, this) {
                Some(p) => {
                    let v: u32 = $to(p.$field);
                    {
                        put(out, v)
                    };
                    OK
                }
                None => E_INVALIDARG,
            })
        }
    };
}

get_space!(get_least, least, f32::to_bits);
get_space!(get_greatest, greatest, f32::to_bits);
get_space!(get_mode, mode, |m: u32| m);

/// `SetAllParameters(DS3DBUFFER*, apply)`: dwSize, vPosition, vVelocity, the cone's two
/// angles, its orientation, its outside volume, flMinDistance, flMaxDistance, dwMode.
extern "system" fn set_all(this: u32, all: u32, _: u32) -> i32 {
    if all == 0 {
        return E_INVALIDARG;
    }
    with(|s| match space(s, this) {
        Some(p) => {
            p.place = [
                float(get(all + 4)),
                float(get(all + 8)),
                float(get(all + 0xC)),
            ];
            p.least = float(get(all + 0x34));
            p.greatest = float(get(all + 0x38));
            p.mode = get(all + 0x3C);
            OK
        }
        None => E_INVALIDARG,
    })
}

static BUFFER_3D: Table<21> = Table(
    [
        no_interface as *const (),
        add_ref as *const (),
        release as *const (),
        zero1 as *const (), // GetAllParameters
        zero2 as *const (), // GetConeAngles
        zero1 as *const (), // GetConeOrientation
        zero1 as *const (), // GetConeOutsideVolume
        get_greatest as *const (),
        get_least as *const (),
        get_mode as *const (),
        get_place as *const (),
        zero1 as *const (), // GetVelocity
        set_all as *const (),
        ok3 as *const (), // SetConeAngles
        ok4 as *const (), // SetConeOrientation
        ok2 as *const (), // SetConeOutsideVolume
        set_greatest as *const (),
        set_least as *const (),
        set_mode as *const (),
        set_place as *const (),
        ok4 as *const (), // SetVelocity
    ],
    std::sync::OnceLock::new(),
);

extern "system" fn set_rolloff(_: u32, value: u32, _: u32) -> i32 {
    with(|s| s.rolloff = float(value));
    OK
}

extern "system" fn get_rolloff(_: u32, out: u32) -> i32 {
    let v = with(|s| s.rolloff);
    put(out, v.to_bits());
    OK
}

static LISTENER: Table<18> = Table(
    [
        no_interface as *const (),
        add_ref as *const (),
        release as *const (),
        zero1 as *const (), // GetAllParameters
        zero1 as *const (), // GetDistanceFactor
        zero1 as *const (), // GetDopplerFactor
        zero2 as *const (), // GetOrientation
        zero1 as *const (), // GetPosition
        get_rolloff as *const (),
        zero1 as *const (), // GetVelocity
        ok2 as *const (),   // SetAllParameters
        ok2 as *const (),   // SetDistanceFactor
        ok2 as *const (),   // SetDopplerFactor
        ok7 as *const (),   // SetOrientation
        ok4 as *const (),   // SetPosition
        set_rolloff as *const (),
        ok4 as *const (), // SetVelocity
        ok0 as *const (), // CommitDeferredSettings
    ],
    std::sync::OnceLock::new(),
);

// ---- the mixer ----

/// Hundredths of a decibel to a gain.
fn gain(centibels: i32) -> f32 {
    10f32.powf(centibels.clamp(-10000, 0) as f32 / 2000.0)
}

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
    let data = b.data.as_ptr();
    let sample = |at: usize| -> f32 {
        unsafe {
            if f.bits == 8 {
                (*data.add(at) as f32 - 128.0) / 128.0
            } else {
                (data.add(at) as *const i16).read_unaligned() as f32 / 32768.0
            }
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

unsafe extern "C" fn feed(
    _: *mut core::ffi::c_void,
    stream: *mut SDL_AudioStream,
    wanted: i32,
    _: i32,
) {
    let frames = (wanted.max(0) / 4) as usize;
    if frames == 0 {
        return;
    }
    let mut out = vec![0f32; frames * 2];
    // DirectSound's focus: while the window is not in front, a buffer made without
    // `DSBCAPS_GLOBALFOCUS` or `DSBCAPS_STICKYFOCUS` plays on unheard (the game asks for
    // neither, so it goes quiet and comes back with the focus).
    let focused = crate::session::FLAGS.window_focused.get() == 1;
    let mut unheard = Vec::new();
    with(|s| {
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
                // scaled twice.
                let m = if b.flags & PRIMARY != 0 { 1.0 } else { master };
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
    unsafe { SDL_PutAudioStreamData(stream, pcm.as_ptr().cast(), (pcm.len() * 2) as i32) };
}

static OPENED: Mutex<bool> = Mutex::new(false);

/// `DirectSoundCreate` for the standalone: the device object into `out`, SDL's audio
/// opened the first time. Fails (as `DirectSoundCreate` can) when SDL cannot.
pub fn direct_sound_create(out: u32) -> i32 {
    let mut opened = OPENED.lock().unwrap();
    if !*opened {
        unsafe {
            if !SDL_InitSubSystem(SDL_INIT_AUDIO) {
                crate::framework::log("platform_sound: SDL audio cannot be started");
                return 0x8878_0078u32 as i32; // DSERR_NODRIVER
            }
            let spec = SDL_AudioSpec {
                format: SDL_AUDIO_S16LE,
                channels: 2,
                freq: OUT_RATE as i32,
            };
            let stream = SDL_OpenAudioDeviceStream(
                SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK,
                &spec,
                Some(feed),
                std::ptr::null_mut(),
            );
            if stream.is_null() {
                crate::framework::log("platform_sound: no SDL audio device");
                return 0x8878_0078u32 as i32;
            }
            SDL_ResumeAudioStreamDevice(stream);
        }
        *opened = true;
    }
    let at = with(|s| new_object(s, DEVICE.start(), Kind::Device));
    put(out, at);
    OK
}
