//! `input::acquire_device` (0x408473): a device record's controller acquired. Only for a
//! listed controller (+0 below `g_controllerCount`) that is attached (its entry's +0x18 1);
//! a record that still has a device (+4) is a fatal error (0x8023831D). The settings: the
//! kind (+8) 2 when 0, 3 for a controller of more than three axes; then the configuration's
//! (each only when set): `dword_4F68E0` 1 .. 3 the kind, `dword_4F68E4` (not negative) the
//! axes' code +0x18, `dword_4F68F8` (up to 100) the strength +0xC, `dword_4F68E8` and
//! `dword_4F68EC` (up to 100) the dead zone +0x10 and the end zone +0x14, `dword_4F68F0`
//! (not negative) +0x1C. Logged (level 2): "JOY:" the number, the axes, the buttons,
//! "axmode=" the kind, "axcode=" +0x18 (hex), "dz=" +0x10 +0x14, "ff=" force feedback, the
//! strength (a line). Then its device: `CreateDevice` with the entry's GUID into +4
//! (0x80238311 "Cannot create joystick object - check directx installation"), its
//! `IDirectInputDevice2A` (0x80238318) in its place (the first given back), the joystick
//! data format (0x80238312), the cooperative level 5 for `hWnd` (0x80238313); the ranges
//! (`DIPROPRANGE` `dword_4F6AB8`): all axes -256 .. 0, the axis at offset (+0x1D & 7) x 4
//! -256 .. 256 and for kinds up to 2 the one at (+0x1C & 7) x 4 too; the dead zones
//! (`DIPROPDWORD` `dword_4F6AA4`): all 0, then +0x10 x 100 by offset with the object left 0
//! (the axis is written into the range's object instead: the x axis gets it, twice for
//! kinds up to 2; section 16); the saturation for all, (100 - dz - ez) x 10000 / (100 - dz)
//! when both are above 0 (signed; unsigned division); the results not looked at. With
//! force feedback (the entry's +0x1C 1) `input::begin_force`; `Acquire`; the readings
//! cleared (`input::clear_device_readings`); +0x24 1 (acquired).
//!
//! In:  esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `input::set_up_device`, `input::device_command`. The effects' functions through
//! `begin_force`'s `part`; the methods the objects' own. Refuses nothing: the fatal errors
//! are raised.

use crate::errors::{BadInput, FatalError, Refused};
use crate::field;
use crate::input::devices::{Deferred, at, put};
use crate::input::di_harness::{JOYSTICK_FORMAT_RVA, a, entry_of, fatal};
use crate::input::guids::INPUT_GUIDS;
use crate::input::store::DEVICES;
use crate::input::types::{
    Controller as C, DiPropDword as W, DiPropHeader, DiPropRange as R, InputDevice as D,
};
use crate::session::Dword;
use crate::session::MACHINE;
use rc2k_core::input::acquire::{
    Acquire, HAS_DEVICE, acquire, axis_offset, dead_zone_value, saturation, two_axes,
};
use std::mem::size_of;

// dword_4F68DC .. dword_4F68F8

const JOY_RVA: &std::ffi::CStr = c"JOY:"; // aJoy "JOY:"
const AXMODE_RVA: &std::ffi::CStr = c"axmode="; // aAxmode "axmode="
const AXCODE_RVA: &std::ffi::CStr = c"axcode="; // aAxcode "axcode="
const DZ_RVA: &std::ffi::CStr = c"dz="; // aDz "dz="
const FF_RVA: &std::ffi::CStr = c"ff="; // aFf "ff="

const DIPROP_RANGE: u32 = 4;
const DIPROP_DEADZONE: u32 = 5;
const DIPROP_SATURATION: u32 = 6;

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
fn settings(record: &D, entry: u32) -> Result<(), BadInput> {
    use rc2k_core::input::acquire::{Config, Settings, settings};
    // Converted: `rc2k_core::input::acquire::settings`.
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
        kind: DEVICES.joy_mode.get(),
        axis_code: DEVICES.joy_code.get(),
        dead_zone: DEVICES.joy_dead_zone.get(),
        end_zone: DEVICES.joy_end_zone.get(),
        axis_pick: DEVICES.alt_axes_code.get(),
        strength: DEVICES.ff_gain.get(),
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
        || late.get(at(entry, field!(C, axes))),
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
    log_int(at(entry, field!(C, axes))?, false);
    log_int(at(entry, field!(C, buttons))?, false);
    log_text(AXMODE_RVA);
    log_int(record.kind.get(), false);
    log_text(AXCODE_RVA);
    crate::logfile::log_hex8::log_hex8(record.axis_code.get(), false, 2);
    log_text(DZ_RVA);
    log_int(record.dead_zone.get(), false);
    log_int(record.end_zone.get(), false);
    log_text(FF_RVA);
    log_int(at(entry, field!(C, force_feedback))?, false);
    log_int(record.strength.get(), true);
    Ok(())
}

/// The axis at offset (the record's axis pick `n` & 7) x 4.
fn axis(record: &D, n: usize) -> u32 {
    axis_offset(record.axis_pick_byte(n))
}

/// The ranges, dead zones and saturation.
fn properties(record: &D) -> Result<(), BadInput> {
    let device = record.device.get();
    let set =
        |what: u32, prop: u32| crate::input::di_objects::set_property(device, what, prop) as u32;
    let (header, object, how) = (
        size_of::<DiPropHeader>() as u32,
        field!(R, header.object),
        field!(R, header.how),
    );
    let range = DEVICES.range_property.addr();
    put(range, field!(R, header.size), size_of::<R>() as u32)?;
    put(range, field!(R, header.header_size), header)?;
    put(range, object, 0)?;
    put(range, how, 0)?;
    put(range, field!(R, min), (-256i32) as u32)?;
    put(range, field!(R, max), 0)?;
    set(DIPROP_RANGE, range);
    put(range, object, axis(record, 1))?;
    put(range, how, 1)?;
    put(range, field!(R, min), (-256i32) as u32)?;
    put(range, field!(R, max), 0x100)?;
    set(DIPROP_RANGE, range);
    // Converted: `rc2k_core::input::acquire` (`axis_offset`, `two_axes`,
    // `dead_zone_value`, `saturation`).
    let two_axes = two_axes(record.kind.get());
    if two_axes {
        put(range, object, axis(record, 0))?;
        set(DIPROP_RANGE, range);
    }
    let dword = DEVICES.dword_property.addr();
    let start = |data: Option<u32>| -> Result<(), BadInput> {
        put(dword, field!(W, header.size), size_of::<W>() as u32)?;
        put(dword, field!(W, header.header_size), header)?;
        put(dword, field!(W, header.object), 0)?;
        put(dword, field!(W, header.how), 0)?;
        if let Some(data) = data {
            put(dword, field!(W, data), data)?;
        }
        Ok(())
    };
    start(Some(0))?;
    set(DIPROP_DEADZONE, dword);
    // (sic) the range's object, not the dead zone's
    put(range, object, axis(record, 1))?;
    put(dword, field!(W, header.how), 1)?;
    put(
        dword,
        field!(W, data),
        dead_zone_value(record.dead_zone.get()),
    )?;
    set(DIPROP_DEADZONE, dword);
    if two_axes {
        put(range, object, axis(record, 0))?;
        set(DIPROP_DEADZONE, dword);
    }
    start(None)?;
    if let Some(data) = saturation(record.dead_zone.get(), record.end_zone.get()) {
        put(dword, field!(W, data), data)?;
        set(DIPROP_SATURATION, dword);
    }
    Ok(())
}

/// The Rust version (see the header).
pub fn acquire_device(record: &D) -> Result<(), Refused> {
    // Converted: `rc2k_core::input::acquire::acquire`.
    let entry = entry_of(record);
    let late = Deferred::default();
    let step = acquire(
        record.controller.get(),
        DEVICES.controller_count.get(),
        || late.get(at(entry, field!(C, attached))),
        || record.device.get(),
    );
    late.finish()?;
    match step {
        Acquire::Nothing => return Ok(()),
        Acquire::HasDevice => return Err(fatal(HAS_DEVICE).into()),
        Acquire::Acquire => {}
    }
    settings(record, entry)?;
    let hr = crate::input::di_objects::create_device(
        DEVICES.direct_input.get(),
        entry + field!(C, instance),
        record.device.addr(),
        0,
    ) as u32;
    if hr != 0 {
        return Err(NO_OBJECT.into());
    }
    let device = record.device.get();
    if crate::input::di_objects::query_interface(
        device,
        INPUT_GUIDS.device2.addr(),
        DEVICES.new_joystick2.addr(),
    ) != 0
    {
        return Err(fatal(0x8023_8318).into());
    }
    crate::input::di_objects::release(device);
    record.device.set(DEVICES.new_joystick2.get());
    DEVICES.new_joystick2.set(0);
    let device = record.device.get();
    if crate::input::di_objects::set_data_format(device, a(JOYSTICK_FORMAT_RVA)) != 0 {
        return Err(fatal(0x8023_8312).into());
    }
    if crate::input::di_objects::set_cooperative_level(device, MACHINE.window.get(), 5) != 0 {
        return Err(fatal(0x8023_8313).into());
    }
    properties(record)?;
    if at(entry, field!(C, force_feedback))? == 1 {
        crate::input::begin_force::begin_force(record)?;
    }
    crate::input::di_objects::acquire(device);
    crate::input::clear_device_readings::clear_device_readings(record);
    record.acquired.set(1);
    Ok(())
}
