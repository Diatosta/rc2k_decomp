//! What they do, as DirectInput 5 does:
//! - `EnumDevices` lists the backend's devices (attached joysticks) to the callback, each a
//!   `DIDEVICEINSTANCEA` (instance and product GUIDs, type 0x104 a joystick, the name), until
//!   it says stop;
//! - `CreateDevice` makes a device of one of them by its instance GUID; `QueryInterface`
//!   for `IID_IDirectInputDevice2A` gives the same object;
//! - a device: `GetCapabilities` (attached, force feedback, axes, buttons, hats),
//!   `EnumObjects` (its axes, by their GUIDs), `SetDataFormat` and `SetCooperativeLevel`
//!   (taken), `SetProperty` (an axis's range, dead zone and saturation, by its offset or for
//!   all; the force feedback's gain and auto-centring), `Acquire`, `Unacquire`, `Poll`,
//!   `GetDeviceState` (a `DIJOYSTATE` of 0x50 bytes: the axes in their ranges with their dead
//!   zones and saturations, the hats, the buttons), `EnumEffects` (the standard effects of a
//!   force-feedback device) and `CreateEffect`;
//! - an effect: `SetParameters` (the `DIEFFECT`'s parts its flags name), `Download`,
//!   `Start`, `Stop`, `Release`: each handed to the backend with the effect as it stands.
//!
//! The objects live as long as their references (`AddRef` / `Release`); every method answers
//! `DI_OK` (0) or a `DIERR_` the way DirectInput would.

use crate::input::types::{
    DiDevCaps, DiDeviceInstance, DiDeviceObjectInstance, DiEffectInfo, DiJoyState,
};
use std::collections::HashMap;
use std::mem::size_of;
use std::sync::Mutex;

// ---- what a backend gives and takes ----

/// A device a backend has.
#[derive(Clone, Debug, Default)]
pub struct DeviceInfo {
    pub name: String,
    pub instance: [u8; 16],
    pub product: [u8; 16],
    /// Its axes as the game's bits: 1 x, 2 y, 4 z, 8 rx, 0x10 ry, 0x20 rz.
    pub axes: u32,
    pub buttons: u32,
    pub hats: u32,
    pub force: bool,
}

/// What a device reads now: the six axes from -32768 to 32767 (x, y, z, rx, ry, rz), the 32
/// buttons as bits, the four hats in hundredths of degrees or `u32::MAX` (centred).
#[derive(Clone, Copy, Debug)]
pub struct Reading {
    pub axes: [i32; 6],
    pub buttons: u32,
    pub hats: [u32; 4],
}

impl Default for Reading {
    fn default() -> Self {
        Self {
            axes: [0; 6],
            buttons: 0,
            hats: [u32::MAX; 4],
        }
    }
}

/// An effect as the game set it: its GUID's last byte's kind (`Kind`), its `DIEFFECT`'s
/// duration (microseconds, `u32::MAX` for ever), gain (0 .. 10000), first direction
/// (hundredths of degrees), envelope (attack level and time, fade level and time) and
/// type-specific dwords.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Effect {
    pub kind: Kind,
    pub duration: u32,
    pub gain: u32,
    pub direction: i32,
    pub envelope: Option<[u32; 4]>,
    pub params: Vec<i32>,
}

/// The standard effects (`GUID_ConstantForce` .. `GUID_Friction`: their GUIDs differ in the
/// first dword, 0x13541C20 ..).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Kind {
    #[default]
    Constant,
    Ramp,
    Square,
    Sine,
    Triangle,
    SawUp,
    SawDown,
    Spring,
    Damper,
    Inertia,
    Friction,
}

const KINDS: [Kind; 11] = [
    Kind::Constant,
    Kind::Ramp,
    Kind::Square,
    Kind::Sine,
    Kind::Triangle,
    Kind::SawUp,
    Kind::SawDown,
    Kind::Spring,
    Kind::Damper,
    Kind::Inertia,
    Kind::Friction,
];

impl Kind {
    /// `DIEFT_`: 1 constant, 2 ramp, 3 periodic, 4 condition.
    fn family(self) -> u32 {
        match self {
            Kind::Constant => 1,
            Kind::Ramp => 2,
            Kind::Square | Kind::Sine | Kind::Triangle | Kind::SawUp | Kind::SawDown => 3,
            _ => 4,
        }
    }

    fn guid(self) -> [u8; 16] {
        let n = KINDS.iter().position(|&k| k == self).unwrap() as u32;
        guid(
            0x1354_1C20 + n,
            0x8E33,
            0x11D0,
            [0x9A, 0xD0, 0x00, 0xA0, 0xC9, 0xA0, 0x6E, 0x35],
        )
    }

    fn of_guid(g: &[u8; 16]) -> Option<Kind> {
        KINDS.iter().copied().find(|k| k.guid() == *g)
    }
}

/// A backend: the devices and their effects (device numbers are places in `devices`).
pub trait Backend: Send {
    fn devices(&mut self) -> Vec<DeviceInfo>;
    fn read(&mut self, device: usize) -> Reading;
    /// The effect `id` (made or changed) on the device; `start` it is started (again) now,
    /// `playing` whether it runs.
    fn effect(&mut self, device: usize, id: u32, effect: &Effect, start: bool, playing: bool);
    fn effect_gone(&mut self, device: usize, id: u32);
    fn gain(&mut self, device: usize, gain: u32);
    fn centring(&mut self, device: usize, on: bool);
    /// The device acquired (true) or given back.
    fn acquired(&mut self, device: usize, on: bool);
}

static BACKEND: Mutex<Option<Box<dyn Backend>>> = Mutex::new(None);

/// The backend from now on.
pub fn set_backend(backend: Box<dyn Backend>) {
    *BACKEND.lock().unwrap_or_else(|e| e.into_inner()) = Some(backend);
}

fn backend<R>(f: impl FnOnce(&mut dyn Backend) -> R) -> Option<R> {
    let mut b = BACKEND.lock().unwrap_or_else(|e| e.into_inner());
    b.as_mut().map(|b| f(b.as_mut()))
}

/// A GUID's 16 bytes from its parts.
pub fn guid(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> [u8; 16] {
    let mut g = [0u8; 16];
    g[0..4].copy_from_slice(&d1.to_le_bytes());
    g[4..6].copy_from_slice(&d2.to_le_bytes());
    g[6..8].copy_from_slice(&d3.to_le_bytes());
    g[8..].copy_from_slice(&d4);
    g
}

/// The axes' GUIDs in the game's bits' order: x, y, z, rx, ry, rz (`GUID_XAxis` ..).
fn axis_guid(n: usize) -> [u8; 16] {
    let d1 = [
        0xA36D_02E0,
        0xA36D_02E1,
        0xA36D_02E2,
        0xA36D_02F4,
        0xA36D_02F5,
        0xA36D_02E3,
    ][n];
    guid(
        d1,
        0xC9F3,
        0x11CF,
        [0xBF, 0xC7, 0x44, 0x45, 0x53, 0x54, 0x00, 0x00],
    )
}

/// `IID_IDirectInputDevice2A`.
fn device2_iid() -> [u8; 16] {
    guid(
        0x5944_E682,
        0xC92E,
        0x11CF,
        [0xBF, 0xC7, 0x44, 0x45, 0x53, 0x54, 0x00, 0x00],
    )
}

pub const DI_OK: i32 = 0;
const DI_NOEFFECT: i32 = 1;
const DIERR_INVALIDPARAM: i32 = 0x8007_0057u32 as i32;
const DIERR_DEVICENOTREG: i32 = 0x8004_0154u32 as i32;
const DIERR_NOTACQUIRED: i32 = 0x8007_000Cu32 as i32;
const DIERR_UNSUPPORTED: i32 = 0x8000_4001u32 as i32;
const E_NOINTERFACE: i32 = 0x8000_4002u32 as i32;

// ---- the objects ----

/// An object as the game sees it: a pointer to its method table (then nothing the game
/// reads). The objects are places in a fixed pool, the lowest free one taken, so that two
/// runs that make the same objects get the same addresses.
#[repr(C)]
struct Object {
    table: *const usize,
    spare: u32,
}

const POOL: usize = 64;

struct Pool(std::cell::UnsafeCell<[Object; POOL]>);

// Safety: a place is written only under `STATE`'s lock, when it is taken; the game reads
// its table pointer.
unsafe impl Sync for Pool {}

static OBJECTS: Pool = Pool(std::cell::UnsafeCell::new(
    [const {
        Object {
            table: std::ptr::null(),
            spare: 0,
        }
    }; POOL],
));

fn pool_base() -> u32 {
    OBJECTS.0.get() as u32
}

/// An axis's range, dead zone and saturation (DirectInput's defaults: 0 .. 65535, 0, 10000).
#[derive(Clone, Copy, Debug)]
struct Axis {
    min: i32,
    max: i32,
    dead: u32,
    saturation: u32,
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            min: 0,
            max: 65535,
            dead: 0,
            saturation: 10000,
        }
    }
}

struct Device {
    device: usize,
    acquired: bool,
    axes: [Axis; 6],
}

struct EffectObject {
    device: usize,
    effect: Effect,
    playing: bool,
}

enum Kind2 {
    Input,
    Device(Device),
    Effect(EffectObject),
}

struct Entry {
    refs: u32,
    what: Kind2,
}

struct State {
    entries: HashMap<u32, Entry>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let s = s.get_or_insert_with(|| State {
        entries: HashMap::new(),
    });
    f(s)
}

/// The object's place from its address (`u32::MAX`: not one of the pool's).
unsafe fn id_of(this: u32) -> u32 {
    let size = std::mem::size_of::<Object>() as u32;
    let offset = this.wrapping_sub(pool_base());
    if offset.is_multiple_of(size) && offset / size < POOL as u32 {
        offset / size
    } else {
        u32::MAX
    }
}

/// A new object, in the lowest free place (0 when the pool is full).
fn make(s: &mut State, table: *const usize, what: Kind2) -> u32 {
    let Some(id) = (0..POOL as u32).find(|n| !s.entries.contains_key(n)) else {
        return 0;
    };
    unsafe { (*OBJECTS.0.get())[id as usize].table = table };
    s.entries.insert(id, Entry { refs: 1, what });
    pool_base() + id * std::mem::size_of::<Object>() as u32
}

/// A new `IDirectInputA` (what `DirectInputCreateA` gives): its address.
pub fn create_input() -> u32 {
    with(|s| make(s, input_table(), Kind2::Input))
}

/// The object at `this` still lives.
fn alive(s: &State, this: u32) -> bool {
    s.entries.contains_key(&unsafe { id_of(this) })
}

unsafe extern "system" fn query(this: u32, iid: *const [u8; 16], out: *mut u32) -> i32 {
    unsafe {
        with(|s| {
            if !alive(s, this) || out.is_null() {
                return DIERR_INVALIDPARAM;
            }
            let id = id_of(this);
            let is_device = matches!(s.entries.get(&id).map(|e| &e.what), Some(Kind2::Device(_)));
            if is_device && !iid.is_null() && *iid == device2_iid() {
                s.entries.get_mut(&id).unwrap().refs += 1;
                *out = this;
                DI_OK
            } else {
                *out = 0;
                E_NOINTERFACE
            }
        })
    }
}

unsafe extern "system" fn add_ref(this: u32) -> u32 {
    with(|s| {
        let id = unsafe { id_of(this) };
        match s.entries.get_mut(&id) {
            Some(e) => {
                e.refs += 1;
                e.refs
            }
            None => 0,
        }
    })
}

unsafe extern "system" fn release(this: u32) -> u32 {
    let gone = with(|s| {
        let id = unsafe { id_of(this) };
        let e = s.entries.get_mut(&id)?;
        e.refs -= 1;
        if e.refs > 0 {
            return Some((e.refs, None));
        }
        let e = s.entries.remove(&id).unwrap();
        Some((
            0,
            match e.what {
                Kind2::Effect(f) => Some((f.device, id)),
                _ => None,
            },
        ))
    });
    match gone {
        Some((refs, effect)) => {
            if let Some((device, id)) = effect {
                backend(|b| b.effect_gone(device, id));
            }
            refs
        }
        None => 0,
    }
}

// IDirectInputA: 0 QueryInterface, 1 AddRef, 2 Release, 3 CreateDevice, 4 EnumDevices,
// 5 GetDeviceStatus, 6 RunControlPanel, 7 Initialize

/// The `DIDEVICEINSTANCEA` of a device.
pub fn device_instance(d: &DeviceInfo) -> DiDeviceInstance {
    let mut name = [0u8; 260];
    let bytes = d.name.as_bytes();
    let n = bytes.len().min(259);
    name[..n].copy_from_slice(&bytes[..n]);
    DiDeviceInstance {
        size: size_of::<DiDeviceInstance>() as u32,
        instance: d.instance,
        product: d.product,
        // DIDEVTYPE_JOYSTICK, a generic one (subtype 1)
        dev_type: 0x104,
        instance_name: name,
        product_name: name,
        ff_driver: [0; 16],
        usage_page: 0,
        usage: 0,
    }
}

unsafe extern "system" fn create_device(
    this: u32,
    guid: *const [u8; 16],
    out: *mut u32,
    _outer: u32,
) -> i32 {
    unsafe {
        if out.is_null() || guid.is_null() {
            return DIERR_INVALIDPARAM;
        }
        let wanted = *guid;
        let devices = backend(|b| b.devices()).unwrap_or_default();
        let Some(device) = devices.iter().position(|d| d.instance == wanted) else {
            *out = 0;
            return DIERR_DEVICENOTREG;
        };
        with(|s| {
            if !alive(s, this) {
                return DIERR_INVALIDPARAM;
            }
            *out = make(
                s,
                device_table(),
                Kind2::Device(Device {
                    device,
                    acquired: false,
                    axes: [Axis::default(); 6],
                }),
            );
            DI_OK
        })
    }
}

type EnumDevicesCallback = unsafe extern "system" fn(u32, u32) -> u32;

unsafe extern "system" fn enum_devices(
    _this: u32,
    kind: u32,
    callback: usize,
    context: u32,
    _flags: u32,
) -> i32 {
    // joysticks only (DIDEVTYPE_JOYSTICK, or all)
    if !(kind == 0 || kind & 0xFF == 4) {
        return DI_OK;
    }
    let devices = backend(|b| b.devices()).unwrap_or_default();
    if callback == 0 {
        return DIERR_INVALIDPARAM;
    }
    let callback: EnumDevicesCallback = unsafe { std::mem::transmute(callback) };
    for d in &devices {
        let instance = device_instance(d);
        if unsafe { callback(&instance as *const DiDeviceInstance as u32, context) } == 0 {
            break;
        }
    }
    DI_OK
}

unsafe extern "system" fn unsupported(_this: u32) -> i32 {
    DIERR_UNSUPPORTED
}

// IDirectInputDevice2A: 0 QueryInterface, 1 AddRef, 2 Release, 3 GetCapabilities,
// 4 EnumObjects, 5 GetProperty, 6 SetProperty, 7 Acquire, 8 Unacquire, 9 GetDeviceState,
// 10 GetDeviceData, 11 SetDataFormat, 12 SetEventNotification, 13 SetCooperativeLevel,
// 14 GetObjectInfo, 15 GetDeviceInfo, 16 RunControlPanel, 17 Initialize, 18 CreateEffect,
// 19 EnumEffects, 20 GetEffectInfo, 21 GetForceFeedbackState, 22 SendForceFeedbackCommand,
// 23 EnumCreatedEffectObjects, 24 Escape, 25 Poll, 26 SendDeviceData

/// The device's number and its backend's information, for the object at `this`.
fn device_of(this: u32) -> Option<(usize, DeviceInfo)> {
    let device = with(
        |s| match s.entries.get(&unsafe { id_of(this) }).map(|e| &e.what) {
            Some(Kind2::Device(d)) => Some(d.device),
            _ => None,
        },
    )?;
    let info = backend(|b| b.devices())?.get(device)?.clone();
    Some((device, info))
}

unsafe extern "system" fn get_capabilities(this: u32, caps: *mut u32) -> i32 {
    let Some((_, d)) = device_of(this) else {
        return DIERR_INVALIDPARAM;
    };
    if caps.is_null() {
        return DIERR_INVALIDPARAM;
    }
    unsafe {
        let full = DiDevCaps {
            size: caps.read_unaligned(),
            flags: 1 | if d.force { 0x100 } else { 0 },
            dev_type: 0x104,
            axes: d.axes.count_ones(),
            buttons: d.buttons,
            hats: d.hats,
            ff_sample_period: 0,
            ff_min_time_resolution: 0,
            firmware_revision: 0,
            hardware_revision: 0,
            ff_driver_version: 0,
        };
        // as much as the caller's size says, its size kept
        let room = (caps.read_unaligned() as usize).min(size_of::<DiDevCaps>());
        if room > 4 {
            let from = (&full as *const DiDevCaps as *const u8).add(4);
            std::ptr::copy_nonoverlapping(from, (caps as *mut u8).add(4), room - 4);
        }
    }
    DI_OK
}

type EnumObjectsCallback = unsafe extern "system" fn(u32, u32) -> u32;

unsafe extern "system" fn enum_objects(
    this: u32,
    callback: usize,
    context: u32,
    _flags: u32,
) -> i32 {
    let Some((_, d)) = device_of(this) else {
        return DIERR_INVALIDPARAM;
    };
    if callback == 0 {
        return DIERR_INVALIDPARAM;
    }
    let callback: EnumObjectsCallback = unsafe { std::mem::transmute(callback) };
    for n in 0..6 {
        if d.axes & (1 << n) == 0 {
            continue;
        }
        let object = DiDeviceObjectInstance {
            size: size_of::<DiDeviceObjectInstance>() as u32,
            guid_type: axis_guid(n),
            // the axis's place in DIJOYSTATE
            offset: 4 * n as u32,
            // an absolute axis, its number
            kind: 2 | (n as u32) << 8,
            flags: 0,
            name: [0; 260],
            rest: [0; 24],
        };
        if unsafe { callback(&object as *const DiDeviceObjectInstance as u32, context) } == 0 {
            break;
        }
    }
    DI_OK
}

unsafe extern "system" fn get_property(_this: u32, _what: u32, _header: u32) -> i32 {
    DIERR_UNSUPPORTED
}

/// The axes a property's header names: by an offset into `DIJOYSTATE` (dwHow 1) or every one
/// (dwHow 0).
fn axes_named(how: u32, object: u32) -> Vec<usize> {
    match how {
        0 => (0..6).collect(),
        1 if object < 0x18 && object.is_multiple_of(4) => vec![(object / 4) as usize],
        _ => vec![],
    }
}

unsafe extern "system" fn set_property(this: u32, what: u32, header: *const u32) -> i32 {
    if header.is_null() {
        return DIERR_INVALIDPARAM;
    }
    let read = |n: usize| unsafe { header.add(n).read_unaligned() };
    let (object, how) = (read(2), read(3));
    let device = with(
        |s| match s.entries.get(&unsafe { id_of(this) }).map(|e| &e.what) {
            Some(Kind2::Device(d)) => Some(d.device),
            _ => None,
        },
    );
    let Some(device) = device else {
        return DIERR_INVALIDPARAM;
    };
    match what {
        // DIPROP_RANGE: lMin, lMax
        4..=6 => with(|s| {
            let Some(Kind2::Device(d)) = s
                .entries
                .get_mut(&unsafe { id_of(this) })
                .map(|e| &mut e.what)
            else {
                return DIERR_INVALIDPARAM;
            };
            let axes = axes_named(how, object);
            if axes.is_empty() {
                return DIERR_INVALIDPARAM;
            }
            for a in axes {
                match what {
                    4 => {
                        d.axes[a].min = read(4) as i32;
                        d.axes[a].max = read(5) as i32;
                    }
                    5 => d.axes[a].dead = read(4).min(10000),
                    _ => d.axes[a].saturation = read(4).min(10000),
                }
            }
            DI_OK
        }),
        // DIPROP_FFGAIN, DIPROP_AUTOCENTER
        7 => {
            backend(|b| b.gain(device, read(4).min(10000)));
            DI_OK
        }
        9 => {
            backend(|b| b.centring(device, read(4) != 0));
            DI_OK
        }
        _ => DIERR_UNSUPPORTED,
    }
}

fn set_acquired(this: u32, on: bool) -> i32 {
    let device = with(|s| {
        match s
            .entries
            .get_mut(&unsafe { id_of(this) })
            .map(|e| &mut e.what)
        {
            Some(Kind2::Device(d)) => {
                let was = d.acquired;
                d.acquired = on;
                Some((d.device, was))
            }
            _ => None,
        }
    });
    match device {
        Some((device, was)) => {
            if was != on {
                backend(|b| b.acquired(device, on));
                DI_OK
            } else {
                DI_NOEFFECT
            }
        }
        None => DIERR_INVALIDPARAM,
    }
}

unsafe extern "system" fn acquire(this: u32) -> i32 {
    set_acquired(this, true)
}

unsafe extern "system" fn unacquire(this: u32) -> i32 {
    set_acquired(this, false)
}

/// A reading of -32768 .. 32767 in an axis's range with its dead zone and saturation, as
/// DirectInput scales it: within the dead zone the centre, past the saturation an end.
fn scaled(raw: i32, a: &Axis) -> i32 {
    let n = (raw as f64 / 32767.0).clamp(-1.0, 1.0);
    let (dead, sat) = (
        a.dead as f64 / 10000.0,
        (a.saturation as f64 / 10000.0).max(a.dead as f64 / 10000.0),
    );
    let magnitude = n.abs();
    let m = if magnitude <= dead {
        0.0
    } else if sat <= dead || magnitude >= sat {
        1.0
    } else {
        (magnitude - dead) / (sat - dead)
    };
    let half = (a.max as f64 - a.min as f64) / 2.0;
    let centre = a.min as f64 + half;
    (centre + n.signum() * m * half).round() as i32
}

unsafe extern "system" fn get_device_state(this: u32, size: u32, out: *mut u8) -> i32 {
    let state = with(
        |s| match s.entries.get(&unsafe { id_of(this) }).map(|e| &e.what) {
            Some(Kind2::Device(d)) => Some((d.device, d.acquired, d.axes)),
            _ => None,
        },
    );
    let Some((device, acquired, axes)) = state else {
        return DIERR_INVALIDPARAM;
    };
    if !acquired {
        return DIERR_NOTACQUIRED;
    }
    if out.is_null() || size < 0x50 {
        return DIERR_INVALIDPARAM;
    }
    let reading = backend(|b| b.read(device)).unwrap_or_default();
    let state = DiJoyState {
        axes: std::array::from_fn(|n| scaled(reading.axes[n], &axes[n])),
        sliders: [0; 2],
        hats: reading.hats,
        buttons: std::array::from_fn(|n| {
            if reading.buttons >> n & 1 != 0 {
                0x80
            } else {
                0
            }
        }),
    };
    unsafe { (out as *mut DiJoyState).write_unaligned(state) };
    DI_OK
}

unsafe extern "system" fn taken2(_this: u32, _a: u32) -> i32 {
    DI_OK
}

unsafe extern "system" fn taken3(_this: u32, _a: u32, _b: u32) -> i32 {
    DI_OK
}

unsafe extern "system" fn poll(this: u32) -> i32 {
    with(
        |s| match s.entries.get(&unsafe { id_of(this) }).map(|e| &e.what) {
            Some(Kind2::Device(d)) if d.acquired => DI_OK,
            Some(Kind2::Device(_)) => DIERR_NOTACQUIRED,
            _ => DIERR_INVALIDPARAM,
        },
    )
}

type EnumEffectsCallback = unsafe extern "system" fn(u32, u32) -> u32;

unsafe extern "system" fn enum_effects(this: u32, callback: usize, context: u32, kind: u32) -> i32 {
    let Some((_, d)) = device_of(this) else {
        return DIERR_INVALIDPARAM;
    };
    if callback == 0 {
        return DIERR_INVALIDPARAM;
    }
    if !d.force {
        return DI_OK;
    }
    let callback: EnumEffectsCallback = unsafe { std::mem::transmute(callback) };
    for k in KINDS {
        // DIEFT_ALL (0) or the kind asked
        if kind & 0xFF != 0 && kind & 0xFF != k.family() {
            continue;
        }
        let mut name = [0u8; 260];
        let text = format!("{k:?}");
        name[..text.len()].copy_from_slice(text.as_bytes());
        let info = DiEffectInfo {
            size: size_of::<DiEffectInfo>() as u32,
            guid: k.guid(),
            eff_type: k.family(),
            // every parameter but the trigger's
            static_params: 0x1E7,
            dynamic_params: 0x1E5,
            name,
        };
        if unsafe { callback(&info as *const DiEffectInfo as u32, context) } == 0 {
            break;
        }
    }
    DI_OK
}

/// The parts of a `DIEFFECT` that `flags` name, into `e`.
unsafe fn take_parameters(e: &mut Effect, at: *const u32, flags: u32) {
    unsafe {
        let read = |n: usize| at.add(n).read_unaligned();
        // DIEP_DURATION 1, DIEP_GAIN 4, DIEP_DIRECTION 0x40, DIEP_ENVELOPE 0x80,
        // DIEP_TYPESPECIFICPARAMS 0x100
        if flags & 1 != 0 {
            e.duration = read(2);
        }
        if flags & 4 != 0 {
            e.gain = read(4).min(10000);
        }
        if flags & 0x40 != 0 {
            let directions = read(9) as *const i32;
            if !directions.is_null() && read(7) > 0 {
                e.direction = directions.read_unaligned();
            }
        }
        if flags & 0x80 != 0 {
            let envelope = read(10) as *const u32;
            e.envelope = (!envelope.is_null()).then(|| {
                // DIENVELOPE: dwSize, dwAttackLevel, dwAttackTime, dwFadeLevel, dwFadeTime
                [1, 2, 3, 4].map(|n| envelope.add(n).read_unaligned())
            });
        }
        if flags & 0x100 != 0 {
            let (size, params) = (read(11) as usize, read(12) as *const i32);
            if !params.is_null() {
                e.params = (0..size / 4)
                    .map(|n| params.add(n).read_unaligned())
                    .collect();
            }
        }
    }
}

const ALL_PARAMETERS: u32 = 0x1FF;
const DIEP_START: u32 = 0x2000_0000;

unsafe extern "system" fn create_effect(
    this: u32,
    guid: *const [u8; 16],
    effect: *const u32,
    out: *mut u32,
    _outer: u32,
) -> i32 {
    unsafe {
        if guid.is_null() || out.is_null() {
            return DIERR_INVALIDPARAM;
        }
        let Some(kind) = Kind::of_guid(&*guid) else {
            *out = 0;
            return DIERR_DEVICENOTREG;
        };
        let Some((device, d)) = device_of(this) else {
            return DIERR_INVALIDPARAM;
        };
        if !d.force {
            *out = 0;
            return DIERR_UNSUPPORTED;
        }
        let mut e = Effect {
            kind,
            duration: u32::MAX,
            gain: 10000,
            ..Default::default()
        };
        if !effect.is_null() {
            take_parameters(&mut e, effect, ALL_PARAMETERS);
        }
        let at = with(|s| {
            make(
                s,
                effect_table(),
                Kind2::Effect(EffectObject {
                    device,
                    effect: e.clone(),
                    playing: false,
                }),
            )
        });
        let id = id_of(at);
        backend(|b| b.effect(device, id, &e, false, false));
        *out = at;
        DI_OK
    }
}

// IDirectInputEffect: 0 QueryInterface, 1 AddRef, 2 Release, 3 Initialize, 4 GetEffectGuid,
// 5 GetParameters, 6 SetParameters, 7 Start, 8 Stop, 9 GetEffectStatus, 10 Download,
// 11 Unload, 12 Escape

/// The effect at `this` changed by `f` (true: it is started, or started again, now), then
/// handed to the backend.
fn change_effect(this: u32, f: impl FnOnce(&mut EffectObject) -> bool) -> i32 {
    let id = unsafe { id_of(this) };
    let now = with(|s| match s.entries.get_mut(&id).map(|e| &mut e.what) {
        Some(Kind2::Effect(e)) => {
            let start = f(e);
            Some((e.device, e.effect.clone(), start, e.playing))
        }
        _ => None,
    });
    match now {
        Some((device, effect, start, playing)) => {
            backend(|b| b.effect(device, id, &effect, start, playing));
            DI_OK
        }
        None => DIERR_INVALIDPARAM,
    }
}

unsafe extern "system" fn set_parameters(this: u32, effect: *const u32, flags: u32) -> i32 {
    if effect.is_null() {
        return DIERR_INVALIDPARAM;
    }
    change_effect(this, |e| {
        unsafe { take_parameters(&mut e.effect, effect, flags) };
        let start = flags & DIEP_START != 0;
        if start {
            e.playing = true;
        }
        start
    })
}

unsafe extern "system" fn start(this: u32, _iterations: u32, _flags: u32) -> i32 {
    change_effect(this, |e| {
        e.playing = true;
        true
    })
}

unsafe extern "system" fn stop(this: u32) -> i32 {
    change_effect(this, |e| {
        e.playing = false;
        false
    })
}

unsafe extern "system" fn download(this: u32) -> i32 {
    change_effect(this, |_| false)
}

unsafe extern "system" fn effect_none(_this: u32) -> i32 {
    DI_OK
}

// ---- the method tables ----

// Each table is filled once (function addresses are not known at compile time) and read
// by the game.

fn input_table() -> *const usize {
    static TABLE: std::sync::OnceLock<[usize; 8]> = std::sync::OnceLock::new();
    TABLE
        .get_or_init(|| {
            [
                query as *const () as usize,
                add_ref as *const () as usize,
                release as *const () as usize,
                create_device as *const () as usize,
                enum_devices as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
            ]
        })
        .as_ptr()
}

fn device_table() -> *const usize {
    static TABLE: std::sync::OnceLock<[usize; 27]> = std::sync::OnceLock::new();
    TABLE
        .get_or_init(|| {
            [
                query as *const () as usize,
                add_ref as *const () as usize,
                release as *const () as usize,
                get_capabilities as *const () as usize,
                enum_objects as *const () as usize,
                get_property as *const () as usize,
                set_property as *const () as usize,
                acquire as *const () as usize,
                unacquire as *const () as usize,
                get_device_state as *const () as usize,
                unsupported as *const () as usize,
                taken2 as *const () as usize,
                taken2 as *const () as usize,
                taken3 as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                create_effect as *const () as usize,
                enum_effects as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                poll as *const () as usize,
                unsupported as *const () as usize,
            ]
        })
        .as_ptr()
}

fn effect_table() -> *const usize {
    static TABLE: std::sync::OnceLock<[usize; 13]> = std::sync::OnceLock::new();
    TABLE
        .get_or_init(|| {
            [
                query as *const () as usize,
                add_ref as *const () as usize,
                release as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                unsupported as *const () as usize,
                set_parameters as *const () as usize,
                start as *const () as usize,
                stop as *const () as usize,
                unsupported as *const () as usize,
                download as *const () as usize,
                effect_none as *const () as usize,
                unsupported as *const () as usize,
            ]
        })
        .as_ptr()
}
