//! `input::update_effects` (0x40F2C0): the race's force feedback brought up to date, once
//! each 40 ms (`input::step_effects_clock`): with sound objects in use
//! (`g_raceSoundObjectCount`), `input::update_car_effects` for player A's car on the first
//! device (`g_inputDevices[0]`, the first view block) if that device has effects; then, if
//! the two devices are one player's (`dword_4F68FC` not 0), for car A again on the second
//! device. Otherwise (no effects on the first device, or one device a player), unless one
//! player (`g_onePlayer` 1), for player B's car on the second device with the second view
//! block. Each only if that device has effects (+0x28 1).
//!
//! In:  nothing
//! Out: eax, ebx, esi, edi as last loaded (the car's number, the view block, the device,
//!      the car's sound object), or as `update_car_effects` leaves them after a call;
//!      flags not read (the caller's `test eax, eax` is inside its `pusha`)
//!
//! See `input/effects.rs`. One caller, `input::step_effects_clock`. Its calls through
//! `part` (`update_car_effects`). Refuses (before anything: the way through is worked out
//! first, nothing it calls changes what decides it) a car number past the eight sound
//! objects and a null device on the way taken. Shadow mode runs the original alone (requests are sent).
//! Self-test: a copy whose fifteen references to the variables go to the test's and whose
//! calls go to the noting stand-in, against the Rust version: every way through.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::input::devices::at;
use crate::input::effects::{CAR, HAS_EFFECTS, Part, compare, dispatch_effects};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::update_effects", rva(0x40F2C0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// g_playerCarA, g_playerCarB, g_onePlayer, g_raceSoundObjectCount (four dwords).
const CARS: usize = 0x514F70;
/// g_inputDevices: the two records' addresses.
const DEVICES: usize = 0x4F6EE8;
/// g_viewBlocks: the two views' blocks.
const VIEWS: usize = 0x63DBB0;
/// g_raceSoundObjects: eight sound objects.
const OBJECTS: usize = 0x512A10;
/// dword_4F68FC: 1 when both devices are player A's.
const TWO_DEVICES: usize = 0x4F68FC;

/// Where the variables are (the game's or a test's).
#[derive(Clone, Copy)]
pub struct Vars {
    pub cars: u32,
    pub devices: u32,
    pub views: u32,
    pub objects: u32,
    pub two_devices: u32,
}

impl Vars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            cars: a(CARS),
            devices: a(DEVICES),
            views: a(VIEWS),
            objects: a(OBJECTS),
            two_devices: a(TWO_DEVICES),
        }
    }
}

fn rd(at: u32) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}

/// The Rust version on the variables in `v`.
pub fn update_effects_with(r: &mut Regs, v: &Vars, part: Part) -> Result<(), Refused> {
    if rd(v.cars + 0xC) == 0 {
        return Ok(());
    }
    // Each load of the registers (car, device, view) and whether its device has effects,
    // in order; worked out before any call is made.
    // A load's car and device checked before its device is read (the original reads the
    // car's sound object and the device's +0x28 with no check: a null device or a car past
    // the eight would fault; 2026-10-19 the test's probe read a null device here).
    let checked = |load: (u32, u32, u32)| -> Result<((u32, u32, u32), bool), Refused> {
        if load.0 >= 8 {
            return Err(BadInput("a car number past the eight sound objects").into());
        }
        if load.1 == 0 {
            return Err(BadInput("a null device on the way taken").into());
        }
        Ok((load, at(load.1, HAS_EFFECTS) == 1))
    };
    let first = checked((rd(v.cars), rd(v.devices), rd(v.views)))?;
    let mut loads = vec![first];
    if first.1 && rd(v.two_devices) != 0 {
        loads.push(checked((rd(v.cars), rd(v.devices + 4), rd(v.views)))?);
    } else if rd(v.cars + 8) != 1 {
        loads.push(checked((
            rd(v.cars + 4),
            rd(v.devices + 4),
            rd(v.views + 4),
        ))?);
    }
    for ((car, device, view), call) in loads {
        r.eax = car;
        r.esi = device;
        r.ebx = view;
        r.edi = rd(v.objects + 4 * car);
        if call {
            part(CAR, r)?;
        }
    }
    Ok(())
}

/// The Rust version.
pub fn update_effects(r: &mut Regs, part: Part) -> Result<(), Refused> {
    update_effects_with(r, &Vars::game(), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effects(&HOOK, r, call_original, update_effects);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cars (4), devices (2), views (2), objects (8), two devices (1)
    let mut mem = [0u32; 17];
    let base = mem.as_mut_ptr() as u32;
    let v = Vars {
        cars: base,
        devices: base + 0x10,
        views: base + 0x18,
        objects: base + 0x20,
        two_devices: base + 0x40,
    };
    // cmp g_raceSoundObjectCount, 0 (its last bytes: the hook's jump covers five)
    let entry = [0x83, 0x3D, 0x7C, 0x4F, 0x51, 0x00, 0x00];
    let copy = match copy_with(
        0x40F2C0,
        0x87,
        &entry,
        &[0x27, 0x53, 0x81],
        &[CAR],
        &[],
        &[
            (CARS, 0x10, v.cars),
            (DEVICES, 8, v.devices),
            (VIEWS, 8, v.views),
            (OBJECTS, 0x20, v.objects),
            (TWO_DEVICES, 4, v.two_devices),
        ],
        15,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    // Two device records of the test's: only +0x28 is read.
    let mut records = [[0u32; 11]; 2];
    let rec = [
        records[0].as_mut_ptr() as u32,
        records[1].as_mut_ptr() as u32,
    ];
    let all = [(base, 0x44usize)];
    for case in 0..64u32 {
        let (car_a, car_b) = match case % 4 {
            0 => (0, 1),
            1 => (3, 7),
            2 => (6, 2),
            _ => (1, 1),
        };
        let fill = || unsafe {
            let m = base as *mut u32;
            m.write_volatile(car_a);
            m.add(1).write_volatile(car_b);
            m.add(2).write_volatile((case >> 2) & 1);
            m.add(3).write_volatile(if case & 8 != 0 { 0 } else { 2 });
            m.add(4).write_volatile(rec[0]);
            m.add(5).write_volatile(rec[1]);
            m.add(6).write_volatile(0x6000_0000);
            m.add(7).write_volatile(0x6000_1000);
            for k in 0..8 {
                m.add(8 + k).write_volatile(0x5000_0000 + 0x100 * k as u32);
            }
            m.add(16).write_volatile((case >> 4) & 1);
            (rec[0] as *mut u32).add(10).write_volatile((case >> 5) & 1);
            (rec[1] as *mut u32)
                .add(10)
                .write_volatile(if case % 3 == 0 { 0 } else { 1 });
        };
        compare(
            &mut t,
            &format!("case {case}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            &all,
            &|r, part| update_effects_with(r, &v, part),
            false,
        );
    }
    // A car past the eight on the way taken is refused (the first load is always car A).
    unsafe {
        (base as *mut u32).write_volatile(9);
        (base as *mut u32).add(3).write_volatile(1);
    }
    let mut r = Regs::SENTINEL;
    t.check(
        update_effects_with(&mut r, &v, &mut crate::sound::turn_parts::test_part).is_err(),
        || "car A 9 was not refused".into(),
    );
    crate::sound::turn_parts::take_called();
    let _ = (&mut records, &mut mem);
    t.finish()
}
