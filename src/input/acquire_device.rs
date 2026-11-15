//! Acquires a device record's controller: applies the configured kind, axes, zones and
//! strength, creates the DirectInput device, sets its data format, cooperative level, axis
//! ranges, dead zones and saturation, starts force feedback when the controller has it,
//! then acquires it and clears its readings.
//!
//! Only a listed, attached controller is acquired; a record that already has a device is a
//! fatal error. The dead-zone loop writes the axis into the range's object field, so the x axis
//! gets the zone (twice for kinds up to 2); this slip is kept as is.

use crate::errors::{BadInput, FatalError, Refused};
use crate::input::devices::Deferred;
use crate::input::di_harness::{entry_of, fatal};
use crate::input::owner::Input;
use crate::input::store::Devices;
use crate::input::types::{DiPropDword as W, DiPropHeader, DiPropRange as R, InputDevice as D};
use crate::memory::session::Dword;
use crate::system::owner::System;
use rc2k_core::input::acquire::{
    Acquire, HAS_DEVICE, acquire, axis_offset, dead_zone_value, saturation, two_axes,
};
use std::mem::size_of;

const JOY_RVA: &std::ffi::CStr = c"JOY:"; // aJoy "JOY:"
const AXMODE_RVA: &std::ffi::CStr = c"axmode="; // aAxmode "axmode="
const AXCODE_RVA: &std::ffi::CStr = c"axcode="; // aAxcode "axcode="
const DZ_RVA: &std::ffi::CStr = c"dz="; // aDz "dz="
const FF_RVA: &std::ffi::CStr = c"ff="; // aFf "ff="

const NO_OBJECT: FatalError = FatalError {
    code: 0x8023_8311,
    message: "Cannot create joystick object - check directx installation",
};

fn log_text(rva: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&rva, false, 2);
}

fn log_int(v: u32, newline: bool) {
    crate::logfile::log_int::log_int(v as i32, newline, 2);
}

/// The settings from the configuration, and the log's line.
fn settings(devices: &Devices, record: &D) -> Result<(), BadInput> {
    use rc2k_core::input::acquire::{Config, Settings, settings};
    let fields = [
        &record.kind,
        &record.axis_code,
        &record.strength,
        &record.dead_zone,
        &record.end_zone,
        &record.axis_pick,
    ];
    let [k, code, strength, dead, end, pick] = fields.map(Dword::get);
    let config = Config {
        kind: devices.joy_mode.get(),
        axis_code: devices.joy_code.get(),
        dead_zone: devices.joy_dead_zone.get(),
        end_zone: devices.joy_end_zone.get(),
        axis_pick: devices.alt_axes_code.get(),
        strength: devices.ff_gain.get(),
    };
    let late = Deferred::default();
    let s = settings(
        Settings {
            kind: k,
            axis_code: code,
            strength,
            dead_zone: dead,
            end_zone: end,
            axis_pick: pick,
        },
        || late.get(entry_of(devices, record).map(|c| c.axes.get())),
        &config,
    );
    late.finish()?;
    let values = [
        s.kind,
        s.axis_code,
        s.strength,
        s.dead_zone,
        s.end_zone,
        s.axis_pick,
    ];
    for (f, v) in fields.into_iter().zip(values) {
        f.set(v);
    }
    log_text(JOY_RVA);
    log_int(record.controller.get(), false);
    log_int(entry_of(devices, record)?.axes.get(), false);
    log_int(entry_of(devices, record)?.buttons.get(), false);
    log_text(AXMODE_RVA);
    log_int(record.kind.get(), false);
    log_text(AXCODE_RVA);
    crate::logfile::log_hex8::log_hex8(record.axis_code.get(), false, 2);
    log_text(DZ_RVA);
    log_int(record.dead_zone.get(), false);
    log_int(record.end_zone.get(), false);
    log_text(FF_RVA);
    log_int(entry_of(devices, record)?.force_feedback.get(), false);
    log_int(record.strength.get(), true);
    Ok(())
}

/// The axis at offset (the record's axis pick `n` & 7) x 4.
fn axis(record: &D, n: usize) -> u32 {
    axis_offset(record.axis_pick_byte(n))
}

/// The ranges, dead zones and saturation.
fn properties(record: &D) {
    use crate::input::di_objects::{Property, set_property};
    let device = record.device.get();
    let header = |object: u32, how: u32| DiPropHeader {
        size: 0,
        header_size: size_of::<DiPropHeader>() as u32,
        object,
        how,
    };
    let range = |object: u32, how: u32, min: i32, max: i32| R {
        header: DiPropHeader {
            size: size_of::<R>() as u32,
            ..header(object, how)
        },
        min,
        max,
    };
    let dword = |how: u32, data: u32| W {
        header: DiPropHeader {
            size: size_of::<W>() as u32,
            ..header(0, how)
        },
        data,
    };
    set_property(device, Property::Range(&range(0, 0, -256, 0)));
    set_property(
        device,
        Property::Range(&range(axis(record, 1), 1, -256, 0x100)),
    );
    let two_axes = two_axes(record.kind.get());
    if two_axes {
        set_property(
            device,
            Property::Range(&range(axis(record, 0), 1, -256, 0x100)),
        );
    }
    set_property(device, Property::DeadZone(&dword(0, 0)));
    // (sic) the dead zone's object stays 0: the range's object was meant
    let dead_zone = dword(1, dead_zone_value(record.dead_zone.get()));
    set_property(device, Property::DeadZone(&dead_zone));
    if two_axes {
        set_property(device, Property::DeadZone(&dead_zone));
    }
    if let Some(data) = saturation(record.dead_zone.get(), record.end_zone.get()) {
        set_property(device, Property::Saturation(&dword(0, data)));
    }
}

/// See the header.
pub fn acquire_device(input: &Input, system: &System, record: &D) -> Result<(), Refused> {
    let late = Deferred::default();
    let step = acquire(
        record.controller.get(),
        input.devices.controller_count.get(),
        || late.get(entry_of(&input.devices, record).map(|c| c.attached.get())),
        || record.device.get(),
    );
    late.finish()?;
    match step {
        Acquire::Nothing => return Ok(()),
        Acquire::HasDevice => return Err(fatal(HAS_DEVICE).into()),
        Acquire::Acquire => {}
    }
    settings(&input.devices, record)?;
    let instance: [u8; 16] = entry_of(&input.devices, record)?.instance.read();
    let made = crate::input::di_objects::create_device(input.devices.direct_input.get(), &instance);
    if crate::input::di_objects::store(&record.device, made) != 0 {
        return Err(NO_OBJECT.into());
    }
    let device = record.device.get();
    let made = crate::input::di_objects::query_interface(device, &input.guids.device2.read());
    if crate::input::di_objects::store(&input.devices.new_joystick2, made) != 0 {
        return Err(fatal(0x8023_8318).into());
    }
    crate::input::di_objects::release(device);
    record.device.set(input.devices.new_joystick2.get());
    input.devices.new_joystick2.set(0);
    let device = record.device.get();
    if crate::input::di_objects::set_data_format(device) != 0 {
        return Err(fatal(0x8023_8312).into());
    }
    if crate::input::di_objects::set_cooperative_level(device, system.session.host.window.get(), 5)
        != 0
    {
        return Err(fatal(0x8023_8313).into());
    }
    properties(record);
    if entry_of(&input.devices, record)?.force_feedback.get() == 1 {
        crate::input::begin_force::begin_force(input, record)?;
    }
    crate::input::di_objects::acquire(device);
    crate::input::clear_device_readings::clear_device_readings(record);
    record.acquired.set(1);
    Ok(())
}
