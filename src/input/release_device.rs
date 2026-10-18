//! `input::release_device` (0x4087F3): a device given back: with a DirectInput device (+4),
//! `Unacquire`, its effects stopped (`sub_4095AE`) when its controller's entry
//! (`dword_4F78E4` + 0x50 its number, +0x1C) is 1, `Release`, +4 0; then +0x24 (acquired)
//! and +0x28 (effects on) 0 and its readings cleared (`input::clear_device_readings`).
//!
//! In:  esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `input::device_command` (three times), the input thread's end. Its calls
//! through `part`: 0 `Unacquire` (eax the device), 1 the effects stopped (esi), 2
//! `Release` (eax); in the game the device's methods and `sub_4095AE` through the game
//! (DirectInput, section 15). Refuses (before anything) a controller number past 16 with
//! a device. Shadow mode runs the original alone (it releases the device). Self-test: a
//! copy whose effects' call goes to a stand-in, against the Rust version, on records of
//! the test's with a device of the test's or none, the controller's entry 1 or not.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::input::clear_device_readings::clear_device_readings;
use crate::input::device_effects::{CONTROLLERS, CONTROLLERS_RVA, controller};
use crate::input::devices::{RECORD_LEN, at, com_call, put, test_record};
use crate::input::poll_device::{NOTED, TestDevice};
use crate::race::mud::copy;
use crate::service::harness::{Region, at as at_rva, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::release_device", rva(0x4087F3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const STOP_EFFECTS_RVA: usize = rva(0x4095AE); // sub_4095AE

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn release_device(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let record = entry.esi;
    let device = at(record, 4);
    if device != 0 {
        let effects = controller(record, 0x1C)? == 1;
        let mut r = Regs {
            eax: device,
            ..*entry
        };
        part(0, &mut r)?;
        if effects {
            let mut e = Regs {
                edi: game_addr(CONTROLLERS_RVA) as u32 + 0x50 * at(record, 0),
                ..r
            };
            part(1, &mut e)?;
        }
        r.eax = device;
        part(2, &mut r)?;
        put(record, 4, 0);
    }
    put(record, 0x24, 0);
    put(record, 0x28, 0);
    clear_device_readings(record);
    Ok(*entry)
}

/// In the game: the device's methods, the effects' stop through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => r.eax = com_call(r.eax, 0x20, &[]),
            1 => call_game(r, game_addr(STOP_EFFECTS_RVA)),
            _ => r.eax = com_call(r.eax, 0x08, &[]),
        }
    }
    Ok(())
}

/// The Rust version with the game's calls: for Rust callers.
pub unsafe fn release_device_in_game(record: u32) -> Result<(), Refused> {
    let r = Regs {
        esi: record,
        ..Regs::default()
    };
    unsafe { release_device(&r, &mut game_part) }.map(|_| ())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { release_device(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The stand-in for the effects' stop: noted as slot 0x95AE.
fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    NOTED.lock().unwrap().push(0x95AE);
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => note(place, r),
        p => game_part(p, r)?,
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp dword ptr [esi+4], 0
    let copy = match copy(
        0x4087F3,
        0x49,
        &[0x60, 0x83, 0x7E, 0x04, 0x00],
        &[0x1F, 0x42],
        &[(0x4095AE, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let entries: Vec<Region> = (0..CONTROLLERS)
        .map(|n| at_rva(CONTROLLERS_RVA + 0x50 * n as usize + 0x1C))
        .collect();
    let saved = capture(&entries);
    let device = TestDevice::new();
    for case in 0..8u32 {
        let mut record = test_record(case);
        let n = case % 3;
        record[0..4].copy_from_slice(&n.to_le_bytes());
        let has = case % 4 != 3;
        record[4..8].copy_from_slice(&(if has { device.address() } else { 0 }).to_le_bytes());
        let start = record.clone();
        let p = record.as_mut_ptr() as u32;
        let effects_on = (case / 2) % 2;
        let run = |original: bool| {
            unsafe {
                std::ptr::copy_nonoverlapping(start.as_ptr(), p as *mut u8, RECORD_LEN);
                game_write(CONTROLLERS_RVA + 0x50 * n as usize + 0x1C, effects_on);
            }
            NOTED.lock().unwrap().clear();
            let before = Regs {
                esi: p,
                ..Regs::SENTINEL
            };
            let kept = if original {
                let mut r = before;
                unsafe { call_game(&mut r, copy.addr()) };
                r.preserved_except(&before, 0)
            } else {
                unsafe { release_device(&before, &mut test_part) }
                    .is_ok_and(|r| r.preserved_except(&before, 0))
            };
            (
                kept,
                std::mem::take(&mut *NOTED.lock().unwrap()),
                capture(&[(p, RECORD_LEN)]),
            )
        };
        let (theirs, mine) = (run(true), run(false));
        t.check(theirs == mine && theirs.0, || {
            format!(
                "case {case}: device {has}, controller {n}, its entry {effects_on}: original {:?} {:x?}; rust {:?} {:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
        drop(record);
    }
    drop(device);
    restore(&entries, &saved);
    t.finish()
}
