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

use crate::errors::{FatalError, Refused};
use crate::field;
use crate::framework::rva;
use crate::input::devices::{at, put};
use crate::input::di_harness::{
    IID_DEVICE2_RVA, JOYSTICK_FORMAT_RVA, a, entry_of, fatal, method, slot,
};
use crate::input::store::DEVICES;
use crate::input::types::{
    Controller as C, DiPropDword as W, DiPropHeader, DiPropRange as R, InputDevice as D,
};
use crate::session::MACHINE;
use crate::store::game_addr;
use rc2k_core::input::acquire::{
    Acquire, HAS_DEVICE, acquire, axis_offset, dead_zone_value, saturation, two_axes,
};
use std::mem::size_of;

// dword_4F68DC .. dword_4F68F8

const JOY_RVA: usize = rva(0x4DA580); // aJoy "JOY:"
const AXMODE_RVA: usize = rva(0x4DA585); // aAxmode "axmode="
const AXCODE_RVA: usize = rva(0x4DA58D); // aAxcode "axcode="
const DZ_RVA: usize = rva(0x4DA595); // aDz "dz="
const FF_RVA: usize = rva(0x4DA599); // aFf "ff="

const DIPROP_RANGE: u32 = 4;
const DIPROP_DEADZONE: u32 = 5;
const DIPROP_SATURATION: u32 = 6;

const NO_OBJECT: FatalError = FatalError {
    code: 0x8023_8311,
    message: "Cannot create joystick object - check directx installation",
};

fn log_text(rva: usize) {
    unsafe { crate::logfile::log_write::log_write(game_addr(rva) as *const u8, false, 2) };
}

fn log_int(v: u32, newline: bool) {
    crate::logfile::log_int::log_int(v as i32, newline, 2);
}

/// The settings from the configuration, and the log's line.
fn settings(record: u32, entry: u32) {
    use rc2k_core::input::acquire::{Config, Settings, settings};
    // Converted: `rc2k_core::input::acquire::settings`.
    let kind = field!(D, kind);
    let fields = [
        kind,
        field!(D, axis_code),
        field!(D, strength),
        field!(D, dead_zone),
        field!(D, end_zone),
        field!(D, axis_pick),
    ];
    let [k, code, strength, dead, end, pick] = fields.map(|f| at(record, f));
    let config = Config {
        kind: DEVICES.joy_mode.get(),
        axis_code: DEVICES.joy_code.get(),
        dead_zone: DEVICES.joy_dead_zone.get(),
        end_zone: DEVICES.joy_end_zone.get(),
        axis_pick: DEVICES.alt_axes_code.get(),
        strength: DEVICES.ff_gain.get(),
    };
    let s = settings(
        Settings {
            kind: k,
            axis_code: code,
            strength,
            dead_zone: dead,
            end_zone: end,
            axis_pick: pick,
        },
        || at(entry, field!(C, axes)),
        &config,
    );
    let values = [
        s.kind,
        s.axis_code,
        s.strength,
        s.dead_zone,
        s.end_zone,
        s.axis_pick,
    ];
    for (f, v) in fields.into_iter().zip(values) {
        put(record, f, v);
    }
    log_text(JOY_RVA);
    log_int(at(record, field!(D, controller)), false);
    log_int(at(entry, field!(C, axes)), false);
    log_int(at(entry, field!(C, buttons)), false);
    log_text(AXMODE_RVA);
    log_int(at(record, kind), false);
    log_text(AXCODE_RVA);
    crate::logfile::log_hex8::log_hex8(at(record, field!(D, axis_code)), false, 2);
    log_text(DZ_RVA);
    log_int(at(record, field!(D, dead_zone)), false);
    log_int(at(record, field!(D, end_zone)), false);
    log_text(FF_RVA);
    log_int(at(entry, field!(C, force_feedback)), false);
    log_int(at(record, field!(D, strength)), true);
}

/// The axis at offset (the record's axis pick `n` & 7) x 4.
fn axis(record: u32, n: u32) -> u32 {
    let at = record.wrapping_add(field!(D, axis_pick) + n);
    axis_offset(unsafe { (at as *const u8).read() })
}

/// The ranges, dead zones and saturation.
fn properties(record: u32) {
    let device = at(record, field!(D, device));
    let set = |what: u32, prop: u32| method(device, slot::SET_PROPERTY, &[what, prop]);
    let (header, object, how) = (
        size_of::<DiPropHeader>() as u32,
        field!(R, header.object),
        field!(R, header.how),
    );
    let range = DEVICES.range_property.addr() as u32;
    put(range, field!(R, header.size), size_of::<R>() as u32);
    put(range, field!(R, header.header_size), header);
    put(range, object, 0);
    put(range, how, 0);
    put(range, field!(R, min), (-256i32) as u32);
    put(range, field!(R, max), 0);
    set(DIPROP_RANGE, range);
    put(range, object, axis(record, 1));
    put(range, how, 1);
    put(range, field!(R, min), (-256i32) as u32);
    put(range, field!(R, max), 0x100);
    set(DIPROP_RANGE, range);
    // Converted: `rc2k_core::input::acquire` (`axis_offset`, `two_axes`,
    // `dead_zone_value`, `saturation`).
    let two_axes = two_axes(at(record, field!(D, kind)));
    if two_axes {
        put(range, object, axis(record, 0));
        set(DIPROP_RANGE, range);
    }
    let dword = DEVICES.dword_property.addr() as u32;
    let start = |data: Option<u32>| {
        put(dword, field!(W, header.size), size_of::<W>() as u32);
        put(dword, field!(W, header.header_size), header);
        put(dword, field!(W, header.object), 0);
        put(dword, field!(W, header.how), 0);
        if let Some(data) = data {
            put(dword, field!(W, data), data);
        }
    };
    start(Some(0));
    set(DIPROP_DEADZONE, dword);
    // (sic) the range's object, not the dead zone's
    put(range, object, axis(record, 1));
    put(dword, field!(W, header.how), 1);
    put(
        dword,
        field!(W, data),
        dead_zone_value(at(record, field!(D, dead_zone))),
    );
    set(DIPROP_DEADZONE, dword);
    if two_axes {
        put(range, object, axis(record, 0));
        set(DIPROP_DEADZONE, dword);
    }
    start(None);
    if let Some(data) = saturation(
        at(record, field!(D, dead_zone)),
        at(record, field!(D, end_zone)),
    ) {
        put(dword, field!(W, data), data);
        set(DIPROP_SATURATION, dword);
    }
}

/// The Rust version (see the header).
pub fn acquire_device(record: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::input::acquire::acquire`.
    let entry = entry_of(record);
    match acquire(
        at(record, field!(D, controller)),
        DEVICES.controller_count.get(),
        || at(entry, field!(C, attached)),
        || at(record, field!(D, device)),
    ) {
        Acquire::Nothing => return Ok(()),
        Acquire::HasDevice => return Err(fatal(HAS_DEVICE).into()),
        Acquire::Acquire => {}
    }
    settings(record, entry);
    let hr = method(
        DEVICES.direct_input.get(),
        slot::CREATE_DEVICE,
        &[
            entry + field!(C, instance),
            record.wrapping_add(field!(D, device)),
            0,
        ],
    );
    if hr != 0 {
        return Err(NO_OBJECT.into());
    }
    let device = at(record, field!(D, device));
    if method(
        device,
        slot::QUERY,
        &[a(IID_DEVICE2_RVA), DEVICES.new_joystick2.addr() as u32],
    ) != 0
    {
        return Err(fatal(0x8023_8318).into());
    }
    method(device, slot::RELEASE, &[]);
    put(record, field!(D, device), DEVICES.new_joystick2.get());
    DEVICES.new_joystick2.set(0);
    let device = at(record, field!(D, device));
    if method(device, slot::SET_DATA_FORMAT, &[a(JOYSTICK_FORMAT_RVA)]) != 0 {
        return Err(fatal(0x8023_8312).into());
    }
    if method(
        device,
        slot::SET_COOPERATIVE_LEVEL,
        &[MACHINE.window.get(), 5],
    ) != 0
    {
        return Err(fatal(0x8023_8313).into());
    }
    properties(record);
    if at(entry, field!(C, force_feedback)) == 1 {
        crate::input::begin_force::begin_force(record)?;
    }
    method(device, slot::ACQUIRE, &[]);
    crate::input::clear_device_readings::clear_device_readings(record);
    put(record, field!(D, acquired), 1);
    Ok(())
}
