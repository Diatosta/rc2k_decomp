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
    DiDevCaps, DiDeviceInstance, DiDeviceObjectInstance, DiEffectInfo, DiJoyState, DiPropDword,
    DiPropRange, Guid,
};
use std::collections::HashMap;
use std::mem::size_of;
use std::sync::Mutex;

// ---- what a backend gives and takes ----

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
/// first dword).
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

pub fn set_backend(backend: Box<dyn Backend>) {
    *BACKEND
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(backend);
}

fn backend<R>(f: impl FnOnce(&mut dyn Backend) -> R) -> Option<R> {
    let mut b = BACKEND
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    b.as_mut().map(|b| f(b.as_mut()))
}

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

/// The most objects at once. An object's handle (what the game holds) is its place in the
/// pool and 1, the lowest free place taken, so that two runs that make the same objects get
/// the same handles (a handle is not an address).
const POOL: usize = 64;

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
    let mut s = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let s = s.get_or_insert_with(|| State {
        entries: HashMap::new(),
    });
    f(s)
}

/// The object's place from its handle (`u32::MAX`: not one of the pool's).
fn id_of(this: u32) -> u32 {
    let place = this.wrapping_sub(1);
    if place < POOL as u32 { place } else { u32::MAX }
}

/// A made object's handle put where the game's method call would have put it: `Ok` its
/// handle into `cell`; a failure but `DIERR_INVALIDPARAM` (which touched nothing) leaves
/// 0 there, as DirectInput's methods do. The `HRESULT`.
pub fn store(cell: &crate::memory::session::Dword, made: Result<u32, i32>) -> i32 {
    match made {
        Ok(handle) => {
            cell.set(handle);
            DI_OK
        }
        Err(hr) => {
            if hr != DIERR_INVALIDPARAM {
                cell.set(0);
            }
            hr
        }
    }
}

/// A new object, in the lowest free place: its handle (0 when the pool is full).
fn make(s: &mut State, what: Kind2) -> u32 {
    let Some(id) = (0..POOL as u32).find(|n| !s.entries.contains_key(n)) else {
        return 0;
    };
    s.entries.insert(id, Entry { refs: 1, what });
    id + 1
}

/// A new `IDirectInputA` (what `DirectInputCreateA` gives): its handle.
pub fn create_input() -> u32 {
    with(|s| make(s, Kind2::Input))
}

fn alive(s: &State, this: u32) -> bool {
    s.entries.contains_key(&id_of(this))
}

/// `QueryInterface`: a device's `IDirectInputDevice2A` (itself, one reference more).
/// The interface's object handle (`Err(E_NOINTERFACE)`: none, the game's out cell 0).
pub fn query_interface(this: u32, iid: &Guid) -> Result<u32, i32> {
    with(|s| {
        if !alive(s, this) {
            return Err(DIERR_INVALIDPARAM);
        }
        let id = id_of(this);
        let is_device = matches!(s.entries.get(&id).map(|e| &e.what), Some(Kind2::Device(_)));
        if is_device && *iid == device2_iid() {
            s.entries.get_mut(&id).unwrap().refs += 1;
            Ok(this)
        } else {
            Err(E_NOINTERFACE)
        }
    })
}

pub fn release(this: u32) -> u32 {
    let gone = with(|s| {
        let id = id_of(this);
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

pub fn device_instance(d: &DeviceInfo) -> DiDeviceInstance {
    let mut name = [0u8; 260];
    let bytes = d.name.as_bytes();
    let n = bytes.len().min(259);
    name[..n].copy_from_slice(&bytes[..n]);
    DiDeviceInstance {
        size: size_of::<DiDeviceInstance>() as u32,
        instance: d.instance,
        product: d.product,
        // DIDEVTYPE_JOYSTICK, a generic one (subtype 1).
        dev_type: 0x104,
        instance_name: name,
        product_name: name,
        ff_driver: [0; 16],
        usage_page: 0,
        usage: 0,
    }
}

/// The device `wanted` (its instance GUID) made: its object handle (`Err(DEVICENOTREG)`
/// when the backend has none; the game's out cell 0 then).
pub fn create_device(this: u32, wanted: &Guid) -> Result<u32, i32> {
    let devices = backend(|b| b.devices()).unwrap_or_default();
    let Some(device) = devices.iter().position(|d| d.instance == *wanted) else {
        return Err(DIERR_DEVICENOTREG);
    };
    with(|s| {
        if !alive(s, this) {
            return Err(DIERR_INVALIDPARAM);
        }
        Ok(make(
            s,
            Kind2::Device(Device {
                device,
                acquired: false,
                axes: [Axis::default(); 6],
            }),
        ))
    })
}

/// `EnumDevices`' callback: its `DIDEVICEINSTANCEA` -> 0 to stop.
pub fn enum_devices(
    _this: u32,
    kind: u32,
    mut callback: impl FnMut(&DiDeviceInstance) -> u32,
    _flags: u32,
) -> i32 {
    // Joysticks only (DIDEVTYPE_JOYSTICK, or all).
    if !(kind == 0 || kind & 0xFF == 4) {
        return DI_OK;
    }
    let devices = backend(|b| b.devices()).unwrap_or_default();
    for d in &devices {
        let instance = device_instance(d);
        if callback(&instance) == 0 {
            break;
        }
    }
    DI_OK
}

/// The device's number and its backend's information, for the object at `this`.
fn device_of(this: u32) -> Option<(usize, DeviceInfo)> {
    let device = with(|s| match s.entries.get(&id_of(this)).map(|e| &e.what) {
        Some(Kind2::Device(d)) => Some(d.device),
        _ => None,
    })?;
    let info = backend(|b| b.devices())?.get(device)?.clone();
    Some((device, info))
}

/// `GetCapabilities` for a caller whose `DIDEVCAPS` is `size` bytes: the capabilities, their
/// size field `size`. DirectInput stores as much of them as `size` says (`caps_bytes`).
pub fn get_capabilities(this: u32, size: u32) -> Result<DiDevCaps, i32> {
    let Some((_, d)) = device_of(this) else {
        return Err(DIERR_INVALIDPARAM);
    };
    Ok(DiDevCaps {
        size,
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
    })
}

/// `EnumObjects`' callback gets each `DIDEVICEOBJECTINSTANCEA` and answers 0 to stop.
pub fn enum_objects(
    this: u32,
    mut callback: impl FnMut(&DiDeviceObjectInstance) -> u32,
    _flags: u32,
) -> i32 {
    let Some((_, d)) = device_of(this) else {
        return DIERR_INVALIDPARAM;
    };
    for n in 0..6 {
        if d.axes & (1 << n) == 0 {
            continue;
        }
        let object = DiDeviceObjectInstance {
            size: size_of::<DiDeviceObjectInstance>() as u32,
            guid_type: axis_guid(n),
            // The axis's place in DIJOYSTATE.
            offset: 4 * n as u32,
            // An absolute axis, its number.
            kind: 2 | (n as u32) << 8,
            flags: 0,
            name: [0; 260],
            rest: [0; 24],
        };
        if callback(&object) == 0 {
            break;
        }
    }
    DI_OK
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

/// A property of a device, as `SetProperty` takes it (`DIPROP_RANGE` 4, `DIPROP_DEADZONE` 5,
/// `DIPROP_SATURATION` 6, `DIPROP_FFGAIN` 7, `DIPROP_AUTOCENTER` 9).
pub enum Property<'a> {
    Range(&'a DiPropRange),
    DeadZone(&'a DiPropDword),
    Saturation(&'a DiPropDword),
    FfGain(&'a DiPropDword),
    AutoCenter(&'a DiPropDword),
}

pub fn set_property(this: u32, property: Property) -> i32 {
    let device = with(|s| match s.entries.get(&id_of(this)).map(|e| &e.what) {
        Some(Kind2::Device(d)) => Some(d.device),
        _ => None,
    });
    let Some(device) = device else {
        return DIERR_INVALIDPARAM;
    };
    match property {
        Property::Range(_) | Property::DeadZone(_) | Property::Saturation(_) => {
            let header = match &property {
                Property::Range(r) => &r.header,
                Property::DeadZone(w) | Property::Saturation(w) => &w.header,
                _ => unreachable!("the arm is the three that name axes"),
            };
            let (object, how) = (header.object, header.how);
            with(|s| {
                let Some(Kind2::Device(d)) = s.entries.get_mut(&id_of(this)).map(|e| &mut e.what)
                else {
                    return DIERR_INVALIDPARAM;
                };
                let axes = axes_named(how, object);
                if axes.is_empty() {
                    return DIERR_INVALIDPARAM;
                }
                for a in axes {
                    match &property {
                        Property::Range(r) => {
                            d.axes[a].min = r.min;
                            d.axes[a].max = r.max;
                        }
                        Property::DeadZone(w) => d.axes[a].dead = w.data.min(10000),
                        Property::Saturation(w) => d.axes[a].saturation = w.data.min(10000),
                        _ => {}
                    }
                }
                DI_OK
            })
        }
        Property::FfGain(w) => {
            backend(|b| b.gain(device, w.data.min(10000)));
            DI_OK
        }
        Property::AutoCenter(w) => {
            backend(|b| b.centring(device, w.data != 0));
            DI_OK
        }
    }
}

fn set_acquired(this: u32, on: bool) -> i32 {
    let device = with(
        |s| match s.entries.get_mut(&id_of(this)).map(|e| &mut e.what) {
            Some(Kind2::Device(d)) => {
                let was = d.acquired;
                d.acquired = on;
                Some((d.device, was))
            }
            _ => None,
        },
    );
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

pub fn acquire(this: u32) -> i32 {
    set_acquired(this, true)
}

pub fn unacquire(this: u32) -> i32 {
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

/// `GetDeviceState` for a buffer of `size` bytes: the `DIJOYSTATE`.
pub fn get_device_state(this: u32, size: u32) -> Result<DiJoyState, i32> {
    let state = with(|s| match s.entries.get(&id_of(this)).map(|e| &e.what) {
        Some(Kind2::Device(d)) => Some((d.device, d.acquired, d.axes)),
        _ => None,
    });
    let Some((device, acquired, axes)) = state else {
        return Err(DIERR_INVALIDPARAM);
    };
    if !acquired {
        return Err(DIERR_NOTACQUIRED);
    }
    if size < 0x50 {
        return Err(DIERR_INVALIDPARAM);
    }
    let reading = backend(|b| b.read(device)).unwrap_or_default();
    Ok(DiJoyState {
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
    })
}

/// `SetDataFormat`: the game asks for `c_dfDIJoystick`, the state
/// `get_device_state` gives, so the format is not passed.
pub fn set_data_format(_this: u32) -> i32 {
    DI_OK
}

/// `SetCooperativeLevel`: SDL's devices are shared.
pub fn set_cooperative_level(_this: u32, _window: u32, _flags: u32) -> i32 {
    DI_OK
}

pub fn poll(this: u32) -> i32 {
    with(|s| match s.entries.get(&id_of(this)).map(|e| &e.what) {
        Some(Kind2::Device(d)) if d.acquired => DI_OK,
        Some(Kind2::Device(_)) => DIERR_NOTACQUIRED,
        _ => DIERR_INVALIDPARAM,
    })
}

/// `EnumEffects`' callback gets each `DIEFFECTINFOA` and answers 0 to stop.
pub fn enum_effects(this: u32, mut callback: impl FnMut(&DiEffectInfo) -> u32, kind: u32) -> i32 {
    let Some((_, d)) = device_of(this) else {
        return DIERR_INVALIDPARAM;
    };
    if !d.force {
        return DI_OK;
    }
    for k in KINDS {
        // DIEFT_ALL (0) or the kind asked.
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
            // Every parameter but the trigger's.
            static_params: 0x1E7,
            dynamic_params: 0x1E5,
            name,
        };
        if callback(&info) == 0 {
            break;
        }
    }
    DI_OK
}

/// What the stand-in takes from a `DIEFFECT`, read out of the game's record by the caller:
/// duration, gain, the first direction (when the record has directions for an axis), the
/// envelope's attack level, attack time, fade level and fade time (when it has one) and the
/// type-specific dwords (when it has them).
#[derive(Clone, Debug, Default)]
pub struct EffectParameters {
    pub duration: u32,
    pub gain: u32,
    pub direction: Option<i32>,
    pub envelope: Option<[u32; 4]>,
    pub type_specific: Option<Vec<i32>>,
}

/// The parts of a `DIEFFECT` that `flags` name, into `e`: DIEP_DURATION 1, DIEP_GAIN 4,
/// DIEP_DIRECTION 0x40, DIEP_ENVELOPE 0x80, DIEP_TYPESPECIFICPARAMS 0x100.
fn take_parameters(e: &mut Effect, p: &EffectParameters, flags: u32) {
    if flags & 1 != 0 {
        e.duration = p.duration;
    }
    if flags & 4 != 0 {
        e.gain = p.gain.min(10000);
    }
    if flags & 0x40 != 0
        && let Some(direction) = p.direction
    {
        e.direction = direction;
    }
    if flags & 0x80 != 0 {
        e.envelope = p.envelope;
    }
    if flags & 0x100 != 0
        && let Some(params) = &p.type_specific
    {
        e.params = params.clone();
    }
}

const ALL_PARAMETERS: u32 = 0x1FF;
const DIEP_START: u32 = 0x2000_0000;

/// An effect of the GUID `guid` made, with `parameters` when the caller gives a `DIEFFECT`:
/// its object handle (`Err(DEVICENOTREG)` for an unknown kind, `Err(UNSUPPORTED)` on a
/// device without force feedback; the game's out cell 0 then).
pub fn create_effect(
    this: u32,
    guid: &Guid,
    parameters: Option<&EffectParameters>,
) -> Result<u32, i32> {
    let Some(kind) = Kind::of_guid(guid) else {
        return Err(DIERR_DEVICENOTREG);
    };
    let Some((device, d)) = device_of(this) else {
        return Err(DIERR_INVALIDPARAM);
    };
    if !d.force {
        return Err(DIERR_UNSUPPORTED);
    }
    let mut e = Effect {
        kind,
        duration: u32::MAX,
        gain: 10000,
        ..Default::default()
    };
    if let Some(p) = parameters {
        take_parameters(&mut e, p, ALL_PARAMETERS);
    }
    let at = with(|s| {
        make(
            s,
            Kind2::Effect(EffectObject {
                device,
                effect: e.clone(),
                playing: false,
            }),
        )
    });
    let id = id_of(at);
    backend(|b| b.effect(device, id, &e, false, false));
    Ok(at)
}

/// The effect at `this` changed by `f` (true: it is started, or started again, now), then
/// handed to the backend.
fn change_effect(this: u32, f: impl FnOnce(&mut EffectObject) -> bool) -> i32 {
    let id = id_of(this);
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

pub fn set_parameters(this: u32, parameters: &EffectParameters, flags: u32) -> i32 {
    change_effect(this, |e| {
        take_parameters(&mut e.effect, parameters, flags);
        let start = flags & DIEP_START != 0;
        if start {
            e.playing = true;
        }
        start
    })
}

pub fn start(this: u32, _iterations: u32, _flags: u32) -> i32 {
    change_effect(this, |e| {
        e.playing = true;
        true
    })
}

pub fn stop(this: u32) -> i32 {
    change_effect(this, |e| {
        e.playing = false;
        false
    })
}

pub fn download(this: u32) -> i32 {
    change_effect(this, |_| false)
}
