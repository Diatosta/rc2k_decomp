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
//! are raised. Shadow mode runs the original alone (it calls DirectInput). Self-test: the
//! original and the Rust version on DirectInput objects made in Rust (`input::di_harness`),
//! the joystick and the wheel listed (`input::list_joysticks`), on copies of the first
//! device's record: each controller, one not listed and one not attached, kinds 0 .. 3,
//! zones of the test's, the configuration's values set or not; the Rust version alone on a
//! record with a device (the fatal error).

use crate::errors::{FatalError, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::input::devices::{RECORD_LEN, RECORDS, at, put};
use crate::input::di_harness::{
    CONTROLLER_COUNT_RVA, DIRECT_INPUT_RVA, DWORD_PROP_RVA, HWND_RVA, IID_DEVICE2_RVA,
    JOYSTICK_FORMAT_RVA, NEW_DEVICE2_RVA, RANGE_PROP_RVA, a, compare, entry_of, fatal, g, method,
    p, regs_text, shared_regions, slot,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::acquire_device", rva(0x408473), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const KIND_RVA: usize = rva(0x4F68E0); // dword_4F68E0: the configuration's kind
const CODE_RVA: usize = rva(0x4F68E4); // dword_4F68E4: its axes' code
const DEAD_RVA: usize = rva(0x4F68E8); // dword_4F68E8: its dead zone
const END_RVA: usize = rva(0x4F68EC); // dword_4F68EC: its end zone
const FLAG_RVA: usize = rva(0x4F68F0); // dword_4F68F0: its +0x1C
const STRENGTH_RVA: usize = rva(0x4F68F8); // dword_4F68F8: its strength
const CONFIG: (usize, usize) = (rva(0x4F68DC), 0x20); // dword_4F68DC .. dword_4F68F8

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
    unsafe { crate::logfile::log_int::log_int(v as i32, newline, 2) };
}

/// The settings from the configuration, and the log's line.
fn settings(record: u32, entry: u32) {
    if at(record, 8) == 0 {
        put(record, 8, 2);
        if at(entry, 8) > 3 {
            put(record, 8, 3);
        }
    }
    let kind = g(KIND_RVA) as i32;
    if kind != 0 && (1..=3).contains(&kind) {
        put(record, 8, kind as u32);
    }
    if (g(CODE_RVA) as i32) >= 0 {
        put(record, 0x18, g(CODE_RVA));
    }
    for (from, to) in [(STRENGTH_RVA, 0xC), (DEAD_RVA, 0x10), (END_RVA, 0x14)] {
        if g(from) <= 100 {
            put(record, to, g(from));
        }
    }
    if (g(FLAG_RVA) as i32) >= 0 {
        put(record, 0x1C, g(FLAG_RVA));
    }
    log_text(JOY_RVA);
    log_int(at(record, 0), false);
    log_int(at(entry, 8), false);
    log_int(at(entry, 0xC), false);
    log_text(AXMODE_RVA);
    log_int(at(record, 8), false);
    log_text(AXCODE_RVA);
    unsafe { crate::logfile::log_hex8::log_hex8(at(record, 0x18), false, 2) };
    log_text(DZ_RVA);
    log_int(at(record, 0x10), false);
    log_int(at(record, 0x14), false);
    log_text(FF_RVA);
    log_int(at(entry, 0x1C), false);
    log_int(at(record, 0xC), true);
}

/// The axis at offset (the record's byte `byte` & 7) x 4.
fn axis(record: u32, byte: u32) -> u32 {
    let b = unsafe { (record.wrapping_add(byte) as *const u8).read() } as u32;
    (b & 7) * 4
}

/// The ranges, dead zones and saturation.
fn properties(record: u32) {
    let device = at(record, 4);
    let set = |what: u32, prop: u32| method(device, slot::SET_PROPERTY, &[what, prop]);
    let range = a(RANGE_PROP_RVA);
    for (n, v) in [0x18u32, 0x10, 0, 0, (-256i32) as u32, 0]
        .into_iter()
        .enumerate()
    {
        put(range, 4 * n as u32, v);
    }
    set(DIPROP_RANGE, range);
    put(range, 8, axis(record, 0x1D));
    put(range, 0xC, 1);
    put(range, 0x10, (-256i32) as u32);
    put(range, 0x14, 0x100);
    set(DIPROP_RANGE, range);
    let two_axes = at(record, 8) <= 2;
    if two_axes {
        put(range, 8, axis(record, 0x1C));
        set(DIPROP_RANGE, range);
    }
    let dword = a(DWORD_PROP_RVA);
    for (n, v) in [0x14u32, 0x10, 0, 0, 0].into_iter().enumerate() {
        put(dword, 4 * n as u32, v);
    }
    set(DIPROP_DEADZONE, dword);
    // (sic) the range's object, not the dead zone's
    put(range, 8, axis(record, 0x1D));
    put(dword, 0xC, 1);
    put(dword, 0x10, at(record, 0x10).wrapping_mul(100));
    set(DIPROP_DEADZONE, dword);
    if two_axes {
        put(range, 8, axis(record, 0x1C));
        set(DIPROP_DEADZONE, dword);
    }
    for (n, v) in [0x14u32, 0x10, 0, 0].into_iter().enumerate() {
        put(dword, 4 * n as u32, v);
    }
    let rest = 100u32.wrapping_sub(at(record, 0x10));
    if (rest as i32) > 0 {
        let live = rest.wrapping_sub(at(record, 0x14));
        if (live as i32) > 0 {
            put(dword, 0x10, live.wrapping_mul(10000) / rest);
            set(DIPROP_SATURATION, dword);
        }
    }
}

/// The Rust version (see the header).
pub fn acquire_device(record: u32, part: crate::input::make_effect::Part) -> Result<(), Refused> {
    if at(record, 0) >= g(CONTROLLER_COUNT_RVA) {
        return Ok(());
    }
    let entry = entry_of(record);
    if at(entry, 0x18) != 1 {
        return Ok(());
    }
    if at(record, 4) != 0 {
        return Err(fatal(0x8023_831D).into());
    }
    settings(record, entry);
    let hr = method(
        g(DIRECT_INPUT_RVA),
        slot::CREATE_DEVICE,
        &[entry + 0x20, record.wrapping_add(4), 0],
    );
    if hr != 0 {
        return Err(NO_OBJECT.into());
    }
    let device = at(record, 4);
    if method(
        device,
        slot::QUERY,
        &[a(IID_DEVICE2_RVA), a(NEW_DEVICE2_RVA)],
    ) != 0
    {
        return Err(fatal(0x8023_8318).into());
    }
    method(device, slot::RELEASE, &[]);
    put(record, 4, g(NEW_DEVICE2_RVA));
    p(NEW_DEVICE2_RVA, 0);
    let device = at(record, 4);
    if method(device, slot::SET_DATA_FORMAT, &[a(JOYSTICK_FORMAT_RVA)]) != 0 {
        return Err(fatal(0x8023_8312).into());
    }
    if method(device, slot::SET_COOPERATIVE_LEVEL, &[g(HWND_RVA), 5]) != 0 {
        return Err(fatal(0x8023_8313).into());
    }
    properties(record);
    if at(entry, 0x1C) == 1 {
        crate::input::begin_force::begin_force(record, part)?;
    }
    method(device, slot::ACQUIRE, &[]);
    crate::input::clear_device_readings::clear_device_readings(record);
    put(record, 0x24, 1);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = acquire_device(r.esi, &mut crate::input::make_effect::game_part) {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let world = crate::input::di_objects::TestWorld::begin(
        crate::input::di_objects::test_devices(),
        Default::default(),
    );
    let start =
        unsafe { std::slice::from_raw_parts(game_addr(RECORDS[0]) as *const u8, RECORD_LEN) }
            .to_vec();
    let mut record = start.clone();
    let rec = record.as_mut_ptr() as u32;
    let mut regions = shared_regions();
    regions.push((rec, RECORD_LEN));
    regions.push((game_addr(CONFIG.0) as u32, CONFIG.1));
    regions.push((a(NEW_DEVICE2_RVA), 4));
    let fill = |controller: u32, kind: u32, zones: (u32, u32), config: bool| {
        let r = rec;
        let start = &start;
        move || {
            p(DIRECT_INPUT_RVA, crate::input::di_objects::create_input());
            p(CONTROLLER_COUNT_RVA, 0);
            unsafe { std::ptr::write_bytes(game_addr(CONFIG.0) as *mut u8, 0, CONFIG.1) };
            crate::input::list_joysticks::list_joysticks();
            // the second entry not attached: a third "controller" that is listed
            if controller == 2 {
                p(CONTROLLER_COUNT_RVA, 3);
                let third = a(crate::input::di_harness::CONTROLLERS_RVA) + 2 * 0x50;
                put(third, 0x18, 0);
            }
            let values: [(usize, u32); 6] = if config {
                [
                    (KIND_RVA, 1),
                    (CODE_RVA, 0x0102),
                    (STRENGTH_RVA, 80),
                    (DEAD_RVA, 10),
                    (END_RVA, 5),
                    (FLAG_RVA, 0),
                ]
            } else {
                [
                    (KIND_RVA, 0),
                    (CODE_RVA, u32::MAX),
                    (STRENGTH_RVA, 101),
                    (DEAD_RVA, 200),
                    (END_RVA, 101),
                    (FLAG_RVA, u32::MAX),
                ]
            };
            for (at, v) in values {
                p(at, v);
            }
            unsafe { std::ptr::copy_nonoverlapping(start.as_ptr(), r as *mut u8, RECORD_LEN) };
            put(r, 0, controller);
            put(r, 4, 0);
            put(r, 8, kind);
            put(r, 0xC, 50);
            put(r, 0x10, zones.0);
            put(r, 0x14, zones.1);
            put(r, 0x18, 0x0103);
            put(r, 0x1C, 0x0204);
        }
    };
    for controller in [0u32, 1, 2, 5] {
        for kind in 0..=3u32 {
            for (zones, config) in [
                ((10, 20), false),
                ((0, 0), false),
                ((60, 50), false),
                ((15, 5), true),
            ] {
                let prepare = fill(controller, kind, zones, config);
                let entry = Regs {
                    esi: rec,
                    ..Regs::SENTINEL
                };
                compare(
                    &mut t,
                    &format!(
                        "controller {controller}, kind {kind}, zones {zones:?}, configured {config}"
                    ),
                    &world,
                    &regions,
                    &prepare,
                    &mut || {
                        let mut r = entry;
                        unsafe { call_original(&mut r) };
                        regs_text(&r)
                    },
                    &mut || {
                        let done = acquire_device(rec, &mut crate::input::make_effect::game_part);
                        format!(
                            "{}{}",
                            regs_text(&entry),
                            if done.is_ok() { "" } else { " refused" }
                        )
                    },
                );
            }
        }
    }
    // a record that still has a device: the Rust version alone
    let saved = crate::service::harness::capture(&regions);
    world.side();
    fill(0, 2, (10, 20), false)();
    put(rec, 4, 0x1234);
    let done = acquire_device(rec, &mut crate::input::make_effect::game_part);
    crate::service::harness::restore(&regions, &saved);
    world.side();
    t.check(done == Err(fatal(0x8023_831D).into()), || {
        format!("a record with a device: rust {done:?}, not fatal error 0x8023831D")
    });
    drop(world);
    drop(record);
    t.finish()
}
