//! The input's game objects as Rust types (`Controller`, `ForceEffect`, `Binding`,
//! `InputDevice`) and DirectInput's that the game fills (`DIJOYSTATE`, `DIEFFECT`,
//! `DIENVELOPE`); the DirectInput ones keep their layouts, checked at compile time.

use crate::memory::session::{Bytes, Dword};
use std::mem::{offset_of, size_of};

pub type Guid = [u8; 16];

/// A listed joystick, one of 16 (`input::joystick_found`).
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Controller {
    /// +0: DIDEVCAPS' flags: 1 attached, 0x100 force feedback.
    pub caps_flags: Dword,
    /// +4: the device's type (DIDEVICEINSTANCEA's).
    pub dev_type: Dword,
    pub axes: Dword,
    pub buttons: Dword,
    pub hats: Dword,
    /// +0x14: which axes it has: 1 x, 2 y, 4 z, 8 rx, 0x10 ry, 0x20 rz (`input::axis_found`).
    pub axis_bits: Dword,
    pub attached: Dword,
    /// +0x1C: 1 with force feedback.
    pub force_feedback: Dword,
    /// +0x20: the device's instance GUID.
    pub instance: Bytes<16>,
    /// +0x30: "(n) name".
    pub name: Bytes<32>,
}

// ---- the device records ----

/// The little-endian dword of `b` at `at`.
#[cfg(test)]
const fn dword_of(b: &[u8], at: usize) -> Dword {
    Dword::new(u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]))
}

/// `N` dwords of `b` from `at`.
#[cfg(test)]
const fn dwords_of<const N: usize>(b: &[u8], at: usize) -> [Dword; N] {
    let mut out = [const { Dword::new(0) }; N];
    let mut k = 0;
    while k < N {
        out[k] = dword_of(b, at + 4 * k);
        k += 1;
    }
    out
}

/// The dwords `ds` written little-endian into `out` from `at`, bits as they are.
fn put_dwords<'a>(out: &mut [u8], at: usize, ds: impl IntoIterator<Item = &'a Dword>) {
    for (k, d) in ds.into_iter().enumerate() {
        out[at + 4 * k..at + 4 * k + 4].copy_from_slice(&d.get().to_le_bytes());
    }
}

/// DirectInput's `DIENVELOPE`.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct DiEnvelope {
    pub size: Dword,
    pub attack_level: Dword,
    pub attack_time: Dword,
    pub fade_level: Dword,
    pub fade_time: Dword,
}

impl DiEnvelope {
    #[cfg(test)]
    const fn at(b: &[u8], at: usize) -> Self {
        Self {
            size: dword_of(b, at),
            attack_level: dword_of(b, at + 4),
            attack_time: dword_of(b, at + 8),
            fade_level: dword_of(b, at + 12),
            fade_time: dword_of(b, at + 16),
        }
    }

    /// The fields written into `out` from `at` (the inverse of `at`).
    fn put(&self, out: &mut [u8], at: usize) {
        let Self {
            size,
            attack_level,
            attack_time,
            fade_level,
            fade_time,
        } = self;
        put_dwords(
            out,
            at,
            [size, attack_level, attack_time, fade_level, fade_time],
        );
    }
}

/// DirectInput's `DIEFFECT` (DirectX 5: 0x34 bytes). Its pointers (`axes`, `directions`,
/// `envelope`, `type_specific`) hold `make_effect::PART_SET` where the game holds addresses; the
/// DirectInput stand-in is given the parts (`effect_objects::parameters_of`) and takes only
/// whether each pointer is set.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct DiEffect {
    pub size: Dword,
    pub flags: Dword,
    pub duration: Dword,
    pub sample_period: Dword,
    pub gain: Dword,
    pub trigger_button: Dword,
    pub trigger_repeat_interval: Dword,
    pub axes_count: Dword,
    pub axes: Dword,
    pub directions: Dword,
    pub envelope: Dword,
    pub type_specific_size: Dword,
    pub type_specific: Dword,
}

impl DiEffect {
    #[cfg(test)]
    const fn at(b: &[u8], at: usize) -> Self {
        Self {
            size: dword_of(b, at),
            flags: dword_of(b, at + 4),
            duration: dword_of(b, at + 8),
            sample_period: dword_of(b, at + 12),
            gain: dword_of(b, at + 16),
            trigger_button: dword_of(b, at + 20),
            trigger_repeat_interval: dword_of(b, at + 24),
            axes_count: dword_of(b, at + 28),
            axes: dword_of(b, at + 32),
            directions: dword_of(b, at + 36),
            envelope: dword_of(b, at + 40),
            type_specific_size: dword_of(b, at + 44),
            type_specific: dword_of(b, at + 48),
        }
    }

    /// The fields written into `out` from `at` (the inverse of `at`).
    fn put(&self, out: &mut [u8], at: usize) {
        let Self {
            size,
            flags,
            duration,
            sample_period,
            gain,
            trigger_button,
            trigger_repeat_interval,
            axes_count,
            axes,
            directions,
            envelope,
            type_specific_size,
            type_specific,
        } = self;
        put_dwords(
            out,
            at,
            [
                size,
                flags,
                duration,
                sample_period,
                gain,
                trigger_button,
                trigger_repeat_interval,
                axes_count,
                axes,
                directions,
                envelope,
                type_specific_size,
                type_specific,
            ],
        );
    }
}

/// One of a device record's six force-feedback effects (`input::make_effect`).
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ForceEffect {
    /// Which of the six change functions is its own (`effect_changes::effect_change`'s number).
    pub change: Dword,
    /// +4: the kind wanted: 1 constant, 2 ramp, 3 periodic, 4 condition.
    pub kind: Dword,
    /// +8: the GUID wanted (by handle: `guids::EffectGuid`).
    pub wanted: Dword,
    /// +0xC: 1 when the device listed one of its kind (`input::effect_found`).
    pub found: Dword,
    /// +0x10: the GUID the device listed.
    pub guid: Bytes<16>,
    /// +0x20, +0x24, +0x28: what the device listed (`DIEFFECTINFOA`).
    pub eff_type: Dword,
    pub static_params: Dword,
    pub dynamic_params: Dword,
    /// +0x2C: a change waiting for the input thread (`input::set_effect`).
    pub pending: Dword,
    /// +0x30: its `IDirectInputEffect`.
    pub object: Dword,
    /// +0x34: the `DIEFFECT` handed to DirectInput.
    pub effect: DiEffect,
    /// +0x68: the type-specific parameters.
    pub type_specific: [Dword; 6],
    /// +0x80: the axes (the `DIEFFECT` points at them).
    pub axes: [Dword; 2],
    /// +0x88: the directions.
    pub directions: [Dword; 2],
    /// +0x90: the envelope.
    pub envelope: DiEnvelope,
}

impl ForceEffect {
    #[cfg(test)]
    const fn at(b: &[u8], at: usize) -> Self {
        let mut guid = [0u8; 16];
        let mut k = 0;
        while k < 16 {
            guid[k] = b[at + 0x10 + k];
            k += 1;
        }
        Self {
            change: dword_of(b, at),
            kind: dword_of(b, at + 4),
            wanted: dword_of(b, at + 8),
            found: dword_of(b, at + 0xC),
            guid: Bytes::new(guid),
            eff_type: dword_of(b, at + 0x20),
            static_params: dword_of(b, at + 0x24),
            dynamic_params: dword_of(b, at + 0x28),
            pending: dword_of(b, at + 0x2C),
            object: dword_of(b, at + 0x30),
            effect: DiEffect::at(b, at + 0x34),
            type_specific: dwords_of(b, at + 0x68),
            axes: dwords_of(b, at + 0x80),
            directions: dwords_of(b, at + 0x88),
            envelope: DiEnvelope::at(b, at + 0x90),
        }
    }

    /// The fields written into `out` from `at` (the inverse of `at`).
    fn put(&self, out: &mut [u8], at: usize) {
        put_dwords(
            out,
            at,
            [&self.change, &self.kind, &self.wanted, &self.found],
        );
        for k in 0..16 {
            out[at + 0x10 + k] = self.guid.get(k);
        }
        put_dwords(
            out,
            at + 0x20,
            [
                &self.eff_type,
                &self.static_params,
                &self.dynamic_params,
                &self.pending,
                &self.object,
            ],
        );
        self.effect.put(out, at + 0x34);
        put_dwords(out, at + 0x68, &self.type_specific);
        put_dwords(out, at + 0x80, &self.axes);
        put_dwords(out, at + 0x88, &self.directions);
        self.envelope.put(out, at + 0x90);
    }
}

/// A control's binding: its key, the axis's place in the record, its sign, a button's bit.
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Binding {
    pub key: Dword,
    /// The axis's place in the record (0x2C, 0x30, 0x34; 0 none).
    pub place: Dword,
    pub sign: Dword,
    pub button: Dword,
}

impl Binding {
    #[cfg(test)]
    const fn at(b: &[u8], at: usize) -> Self {
        Self {
            key: dword_of(b, at),
            place: dword_of(b, at + 4),
            sign: dword_of(b, at + 8),
            button: dword_of(b, at + 12),
        }
    }

    /// The fields written into `out` from `at` (the inverse of `at`).
    fn put(&self, out: &mut [u8], at: usize) {
        put_dwords(out, at, [&self.key, &self.place, &self.sign, &self.button]);
    }
}

/// A player's input device, one of two (`input/devices.rs`).
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct InputDevice {
    /// +0: the controller's number in the controllers' table.
    pub controller: Dword,
    /// +4: its `IDirectInputDevice2A` (0: none).
    pub device: Dword,
    /// +8: 1 digital, 2 one axis for throttle and brake, 3 an axis each.
    pub kind: Dword,
    /// +0xC: the force feedback's strength (0 .. 100).
    pub strength: Dword,
    pub dead_zone: Dword,
    pub end_zone: Dword,
    /// +0x18: which state dwords the axes are, a byte each (+0x1A, +0x1B the second and
    /// third).
    pub axis_code: Dword,
    /// +0x1C: the axes for the ranges and dead zones, a byte each (+0x1C, +0x1D).
    pub axis_pick: Dword,
    pub sensitivity: Dword,
    /// +0x24: 1 while acquired.
    pub acquired: Dword,
    /// +0x28: 1 while its effects run.
    pub effects_on: Dword,
    /// +0x2C: the three axes, copied from the device's state at each poll.
    pub axes: [Dword; 3],
    /// +0x38: its buttons (bits).
    pub buttons: Dword,
    /// +0x3C: the six effects.
    pub effects: [ForceEffect; 6],
    /// +0x414: 1 while it is read.
    pub reading: Dword,
    pub hat_kind: Dword,
    /// +0x41C: the twelve bindings.
    pub bindings: [Binding; 12],
    /// +0x4DC: steering, brake, throttle (`drive::device_axes`).
    pub readings: [Dword; 3],
    pub bits: Dword,
    /// +0x4EC, +0x4F0, +0x4F4: the tyre change's gauge, stage and clock
    /// (`race::tyre_change_gauge`).
    pub gauge: Dword,
    pub stage: Dword,
    pub clock: Dword,
}

impl InputDevice {
    /// A record from the image's `RECORD_LEN` bytes.
    #[cfg(test)]
    pub const fn from_bytes(b: &[u8; 0x4F8]) -> Self {
        let mut effects = [const { ForceEffect::at(&[0; 0xA4], 0) }; 6];
        let mut k = 0;
        while k < 6 {
            effects[k] = ForceEffect::at(b, 0x3C + 0xA4 * k);
            k += 1;
        }
        let mut bindings = [const { Binding::at(&[0; 16], 0) }; 12];
        let mut k = 0;
        while k < 12 {
            bindings[k] = Binding::at(b, 0x41C + 16 * k);
            k += 1;
        }
        Self {
            controller: dword_of(b, 0),
            device: dword_of(b, 4),
            kind: dword_of(b, 8),
            strength: dword_of(b, 0xC),
            dead_zone: dword_of(b, 0x10),
            end_zone: dword_of(b, 0x14),
            axis_code: dword_of(b, 0x18),
            axis_pick: dword_of(b, 0x1C),
            sensitivity: dword_of(b, 0x20),
            acquired: dword_of(b, 0x24),
            effects_on: dword_of(b, 0x28),
            axes: dwords_of(b, 0x2C),
            buttons: dword_of(b, 0x38),
            effects,
            reading: dword_of(b, 0x414),
            hat_kind: dword_of(b, 0x418),
            bindings,
            readings: dwords_of(b, 0x4DC),
            bits: dword_of(b, 0x4E8),
            gauge: dword_of(b, 0x4EC),
            stage: dword_of(b, 0x4F0),
            clock: dword_of(b, 0x4F4),
        }
    }

    /// The record's `RECORD_LEN` bytes, read from its fields (the inverse of `from_bytes`).
    pub fn to_bytes(&self) -> [u8; 0x4F8] {
        let mut b = [0u8; 0x4F8];
        put_dwords(
            &mut b,
            0,
            [
                &self.controller,
                &self.device,
                &self.kind,
                &self.strength,
                &self.dead_zone,
                &self.end_zone,
                &self.axis_code,
                &self.axis_pick,
                &self.sensitivity,
                &self.acquired,
                &self.effects_on,
            ],
        );
        put_dwords(&mut b, 0x2C, &self.axes);
        put_dwords(&mut b, 0x38, [&self.buttons]);
        for (k, effect) in self.effects.iter().enumerate() {
            effect.put(&mut b, 0x3C + 0xA4 * k);
        }
        put_dwords(&mut b, 0x414, [&self.reading, &self.hat_kind]);
        for (k, binding) in self.bindings.iter().enumerate() {
            binding.put(&mut b, 0x41C + 16 * k);
        }
        put_dwords(&mut b, 0x4DC, &self.readings);
        put_dwords(
            &mut b,
            0x4E8,
            [&self.bits, &self.gauge, &self.stage, &self.clock],
        );
        b
    }

    /// Byte `n` of `axis_code`.
    pub(super) fn axis_code_byte(&self, n: usize) -> u8 {
        self.axis_code.get().to_le_bytes()[n]
    }

    /// Byte `n` of `axis_pick`.
    pub(super) fn axis_pick_byte(&self, n: usize) -> u8 {
        self.axis_pick.get().to_le_bytes()[n]
    }
}

/// DirectInput's `DIJOYSTATE`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct DiJoyState {
    pub axes: [i32; 6],
    pub sliders: [i32; 2],
    pub hats: [u32; 4],
    pub buttons: [u8; 32],
}

impl DiJoyState {
    /// The state as the device state's dwords.
    pub fn dwords(&self) -> [u32; 20] {
        let mut out = [0u32; 20];
        let head = self
            .axes
            .iter()
            .map(|&v| v as u32)
            .chain(self.sliders.iter().map(|&v| v as u32))
            .chain(self.hats);
        for (place, v) in out.iter_mut().zip(head) {
            *place = v;
        }
        for (place, b) in out[12..].iter_mut().zip(self.buttons.as_chunks::<4>().0) {
            *place = u32::from_le_bytes(*b);
        }
        out
    }
}

const _: () = assert!(size_of::<DiEffect>() == 0x34);
const _: () = assert!(size_of::<DiJoyState>() == 0x50);

/// DirectInput's `DIDEVCAPS`.
#[repr(C)]
pub(super) struct DiDevCaps {
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
pub(super) struct DiPropHeader {
    pub size: u32,
    pub header_size: u32,
    pub object: u32,
    pub how: u32,
}

/// DirectInput's `DIPROPDWORD`.
#[repr(C)]
pub(super) struct DiPropDword {
    pub header: DiPropHeader,
    pub data: u32,
}

/// DirectInput's `DIPROPRANGE`.
#[repr(C)]
pub(super) struct DiPropRange {
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
