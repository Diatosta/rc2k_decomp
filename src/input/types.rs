//! The input's game objects as Rust types, the same as IDA's structs of the same names
//! (`Controller`, `ForceEffect`, `Binding`, `InputDevice`) and DirectInput's that the game
//! fills (`DIJOYSTATE`, `DIEFFECT`, `DIENVELOPE`): their layouts, checked at compile time.
//! The code reads and writes the game's memory through `field!` (a field's offset).

use std::mem::{offset_of, size_of};

/// A field's offset in a type, as the `u32` the record helpers take.
#[macro_export]
macro_rules! field {
    ($t:ty, $($f:tt)+) => {
        ::std::mem::offset_of!($t, $($f)+) as u32
    };
}

pub type Guid = [u8; 16];

/// `g_controllers` (16 of them): a listed joystick (`input::joystick_found`).
#[repr(C)]
pub struct Controller {
    /// DIDEVCAPS' flags: 1 attached, 0x100 force feedback.
    pub caps_flags: u32,
    /// The device's type (DIDEVICEINSTANCEA's).
    pub dev_type: u32,
    pub axes: u32,
    pub buttons: u32,
    pub hats: u32,
    /// Which axes it has: 1 x, 2 y, 4 z, 8 rx, 0x10 ry, 0x20 rz (`input::axis_found`).
    pub axis_bits: u32,
    pub attached: u32,
    pub force_feedback: u32,
    pub instance: Guid,
    /// "(n) name".
    pub name: [u8; 32],
}

/// DirectInput's `DIENVELOPE`.
#[repr(C)]
pub struct DiEnvelope {
    pub size: u32,
    pub attack_level: u32,
    pub attack_time: u32,
    pub fade_level: u32,
    pub fade_time: u32,
}

/// DirectInput's `DIEFFECT` (DirectX 5: 0x34 bytes).
#[repr(C)]
pub struct DiEffect {
    pub size: u32,
    pub flags: u32,
    pub duration: u32,
    pub sample_period: u32,
    pub gain: u32,
    pub trigger_button: u32,
    pub trigger_repeat_interval: u32,
    pub axes_count: u32,
    pub axes: u32,
    pub directions: u32,
    pub envelope: u32,
    pub type_specific_size: u32,
    pub type_specific: u32,
}

/// One of a device record's six force-feedback effects (`input::make_effect`).
#[repr(C)]
pub struct ForceEffect {
    /// Its function (`input::effect_N_change`), called with a change.
    pub change: u32,
    /// The kind wanted: 1 constant, 2 ramp, 3 periodic, 4 condition.
    pub kind: u32,
    /// The GUID wanted (a pointer).
    pub wanted: u32,
    /// 1 when the device listed one of its kind (`input::effect_found`).
    pub found: u32,
    pub guid: Guid,
    pub eff_type: u32,
    pub static_params: u32,
    pub dynamic_params: u32,
    /// A change waiting for the input thread (`input::set_effect`).
    pub pending: u32,
    /// Its `IDirectInputEffect`.
    pub object: u32,
    pub effect: DiEffect,
    pub type_specific: [u32; 6],
    pub axes: [u32; 2],
    pub directions: [i32; 2],
    pub envelope: DiEnvelope,
}

/// A control's binding: its key, the axis's place in the record, its sign, a button's bit.
#[repr(C)]
pub struct Binding {
    pub key: u32,
    pub place: u32,
    pub sign: u32,
    pub button: u32,
}

/// `g_inputRecord1` / `2`: a player's input device (`input/devices.rs`).
#[repr(C)]
pub struct InputDevice {
    /// The controller's number in `g_controllers`.
    pub controller: u32,
    /// Its `IDirectInputDevice2A` (0: none).
    pub device: u32,
    /// 1 digital, 2 one axis for throttle and brake, 3 an axis each.
    pub kind: u32,
    /// The force feedback's strength (0 .. 100).
    pub strength: u32,
    pub dead_zone: u32,
    pub end_zone: u32,
    /// Which state dwords the axes are (+0x1A, +0x1B the second and third).
    pub axis_code: [u8; 4],
    /// The axes for the ranges and dead zones (+0x1C, +0x1D).
    pub axis_pick: [u8; 4],
    pub sensitivity: u32,
    pub acquired: u32,
    pub effects_on: u32,
    pub axes: [i32; 3],
    pub buttons: u32,
    pub effects: [ForceEffect; 6],
    /// 1 while it is read.
    pub reading: u32,
    pub hat_kind: u32,
    pub bindings: [Binding; 12],
    /// Steering, brake, throttle (`drive::device_axes`).
    pub readings: [i32; 3],
    pub bits: u32,
    pub rest: [u8; 12],
}

/// DirectInput's `DIJOYSTATE` (`g_deviceState`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DiJoyState {
    pub axes: [i32; 6],
    pub sliders: [i32; 2],
    pub hats: [u32; 4],
    pub buttons: [u8; 32],
}

const _: () = assert!(size_of::<Controller>() == 0x50);
const _: () = assert!(offset_of!(Controller, instance) == 0x20);
const _: () = assert!(size_of::<DiEffect>() == 0x34);
const _: () = assert!(size_of::<ForceEffect>() == 0xA4);
const _: () = assert!(offset_of!(ForceEffect, object) == 0x30);
const _: () = assert!(offset_of!(ForceEffect, effect) == 0x34);
const _: () = assert!(offset_of!(ForceEffect, type_specific) == 0x68);
const _: () = assert!(offset_of!(ForceEffect, axes) == 0x80);
const _: () = assert!(offset_of!(ForceEffect, directions) == 0x88);
const _: () = assert!(offset_of!(ForceEffect, envelope) == 0x90);
const _: () = assert!(size_of::<InputDevice>() == 0x4F8);
const _: () = assert!(offset_of!(InputDevice, sensitivity) == 0x20);
const _: () = assert!(offset_of!(InputDevice, axes) == 0x2C);
const _: () = assert!(offset_of!(InputDevice, effects) == 0x3C);
const _: () = assert!(offset_of!(InputDevice, reading) == 0x414);
const _: () = assert!(offset_of!(InputDevice, bindings) == 0x41C);
const _: () = assert!(offset_of!(InputDevice, readings) == 0x4DC);
const _: () = assert!(offset_of!(InputDevice, bits) == 0x4E8);
const _: () = assert!(size_of::<DiJoyState>() == 0x50);

/// DirectInput's `DIDEVCAPS` (`g_joystickCaps`).
#[repr(C)]
pub struct DiDevCaps {
    pub size: u32,
    pub flags: u32,
    pub dev_type: u32,
    pub axes: u32,
    pub buttons: u32,
    pub hats: u32,
    pub ff_sample_period: u32,
    pub ff_min_time_resolution: u32,
    pub firmware_revision: u32,
    pub hardware_revision: u32,
    pub ff_driver_version: u32,
}

/// DirectInput's `DIPROPHEADER`: the property's size, the header's, the object, how it
/// is named (0 the device, 1 by offset).
#[repr(C)]
pub struct DiPropHeader {
    pub size: u32,
    pub header_size: u32,
    pub object: u32,
    pub how: u32,
}

/// DirectInput's `DIPROPDWORD` (`g_dwordProperty`).
#[repr(C)]
pub struct DiPropDword {
    pub header: DiPropHeader,
    pub data: u32,
}

/// DirectInput's `DIPROPRANGE` (`g_rangeProperty`).
#[repr(C)]
pub struct DiPropRange {
    pub header: DiPropHeader,
    pub min: i32,
    pub max: i32,
}

/// DirectInput's `DIDEVICEINSTANCEA` (`EnumDevices`' callback's).
#[repr(C)]
pub struct DiDeviceInstance {
    pub size: u32,
    pub instance: Guid,
    pub product: Guid,
    pub dev_type: u32,
    pub instance_name: [u8; 260],
    pub product_name: [u8; 260],
    pub ff_driver: Guid,
    pub usage_page: u16,
    pub usage: u16,
}

/// DirectInput's `DIDEVICEOBJECTINSTANCEA` (`EnumObjects`' callback's), its first part.
#[repr(C)]
pub struct DiDeviceObjectInstance {
    pub size: u32,
    pub guid_type: Guid,
    pub offset: u32,
    pub kind: u32,
    pub flags: u32,
    pub name: [u8; 260],
    pub rest: [u8; 24],
}

/// DirectInput's `DIEFFECTINFOA` (`EnumEffects`' callback's).
#[repr(C)]
pub struct DiEffectInfo {
    pub size: u32,
    pub guid: Guid,
    pub eff_type: u32,
    pub static_params: u32,
    pub dynamic_params: u32,
    pub name: [u8; 260],
}

const _: () = assert!(size_of::<DiDevCaps>() == 0x2C);
const _: () = assert!(size_of::<DiPropDword>() == 0x14);
const _: () = assert!(size_of::<DiPropRange>() == 0x18);
const _: () = assert!(size_of::<DiDeviceInstance>() == 0x244);
const _: () = assert!(offset_of!(DiDeviceInstance, instance_name) == 0x28);
const _: () = assert!(size_of::<DiDeviceObjectInstance>() == 0x13C);
const _: () = assert!(size_of::<DiEffectInfo>() == 0x124);
