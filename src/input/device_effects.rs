//! `input::device_effects` (0x4096B4): a device's force-feedback effects told their
//! changes: with its effects on (+0x28 1), a DirectInput device (+4) and its controller's
//! entry (`dword_4F78E4` + 0x50 its number, +0x1C) 1, each of its six effects (from +0x3C,
//! 0xA4 bytes each) that is on (+0x30) and has a change waiting (+0x2C, taken and 0): its
//! function (+0) called with eax the change's low byte, ecx bits 8 .. 19 and edx bits 20 ..
//! 31 (each signed), esi the record and edi the effect.
//!
//! In:  esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::input_tick` (each device, each tick). The effects' functions through
//! `part` (eax .. edi as above; in the game the function's address through the game:
//! DirectInput's, section 15). Refuses (before anything) a controller number past the
//! table's 16. Self-test: a copy whose calls go to stand-ins can not be made (the calls
//! are through the effects' pointers): the original and the Rust version on records of
//! the test's whose effects' functions are a noting stand-in of the test's.

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_read, hook_asm, rva, shadow};
use crate::input::devices::{RECORD_LEN, at, put, test_record};
use crate::input::types::{ForceEffect, InputDevice};
use crate::service::harness::{Region, at as at_rva, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::device_effects", rva(0x4096B4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CONTROLLERS_RVA: usize = rva(0x4F78E4); // dword_4F78E4: 16 entries of 0x50
pub const CONTROLLERS: u32 = 16;
const EFFECTS: u32 = 0x3C;
const EFFECT_LEN: u32 = 0xA4;

/// A call by place: 0 an effect's function (`[edi]`, the registers as the header says).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The controller entry's dword at `offset` for the record's controller, or the refusal.
pub fn controller(record: u32, offset: usize) -> Result<u32, Refused> {
    let n = at(record, field!(InputDevice, controller));
    if n >= CONTROLLERS {
        return Err(BadInput("a controller number past the table's 16").into());
    }
    Ok(unsafe { game_read(CONTROLLERS_RVA + 0x50 * n as usize + offset) })
}

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn device_effects(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let record = entry.esi;
    if at(record, field!(InputDevice, effects_on)) != 1
        || at(record, field!(InputDevice, device)) == 0
    {
        return Ok(*entry);
    }
    if controller(record, 0x1C)? != 1 {
        return Ok(*entry);
    }
    for k in 0..6 {
        let effect = record + EFFECTS + EFFECT_LEN * k;
        if at(effect, field!(ForceEffect, object)) == 0 {
            continue;
        }
        let change = at(effect, field!(ForceEffect, pending));
        put(effect, field!(ForceEffect, pending), 0);
        if change == 0 {
            continue;
        }
        // Converted: `rc2k_core::input::effects::change`.
        let (what, a, b) = rc2k_core::input::effects::change(change);
        let mut r = Regs {
            eax: what,
            ecx: a as u32,
            edx: b as u32,
            esi: record,
            edi: effect,
            ..*entry
        };
        part(0, &mut r)?;
    }
    Ok(*entry)
}

/// In the game: the effect's function (`[edi]`) through the game.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    let function = at(r.edi, 0);
    unsafe { call_game(r, function as usize) };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { device_effects(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

static NOTED: Mutex<Vec<[u32; 5]>> = Mutex::new(Vec::new());

/// The test's effect function: notes eax, ecx, edx, esi, edi; keeps every register.
#[unsafe(naked)]
unsafe extern "C" fn test_effect() {
    core::arch::naked_asm!(
        "pushad",
        "push edi", "push esi", "push edx", "push ecx", "push eax",
        "call {note}",
        "add esp, 20",
        "popad",
        "ret",
        note = sym note_effect,
    )
}

extern "C" fn note_effect(eax: u32, ecx: u32, edx: u32, esi: u32, edi: u32) {
    NOTED.lock().unwrap().push([eax, ecx, edx, esi, edi]);
}

fn test_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    note_effect(r.eax, r.ecx, r.edx, r.esi, r.edi);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry_of = |n: u32| CONTROLLERS_RVA + 0x50 * n as usize + 0x1C;
    let kept: Vec<Region> = (0..CONTROLLERS).map(|n| at_rva(entry_of(n))).collect();
    let saved = capture(&kept);
    for case in 0..16u32 {
        let mut record = test_record(case);
        let mut put_r = |o: usize, v: u32| record[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let controller_n = case % 4;
        put_r(0, controller_n);
        put_r(4, if case % 6 == 5 { 0 } else { 0x1234 });
        put_r(0x28, if case % 7 == 6 { 0 } else { 1 });
        for k in 0..6usize {
            let e = 0x3C + 0xA4 * k;
            put_r(e, test_effect as *const () as u32);
            put_r(e + 0x30, !(case + k as u32).is_multiple_of(3) as u32);
            put_r(
                e + 0x2C,
                if (case + k as u32).is_multiple_of(4) {
                    0
                } else {
                    (case + 1).wrapping_mul(0x9E37_79B9) ^ k as u32
                },
            );
        }
        let start = record.clone();
        let p = record.as_mut_ptr() as u32;
        let run = |original: bool| {
            unsafe {
                std::ptr::copy_nonoverlapping(start.as_ptr(), p as *mut u8, RECORD_LEN);
                crate::framework::game_write(entry_of(controller_n), (case % 5 != 4) as u32);
            }
            NOTED.lock().unwrap().clear();
            let before = Regs {
                esi: p,
                ..Regs::SENTINEL
            };
            let kept_regs = if original {
                let mut r = before;
                unsafe { call_original(&mut r) };
                r.preserved_except(&before, 0)
            } else {
                unsafe { device_effects(&before, &mut test_part) }
                    .is_ok_and(|r| r.preserved_except(&before, 0))
            };
            (
                kept_regs,
                std::mem::take(&mut *NOTED.lock().unwrap()),
                capture(&[(p, RECORD_LEN)]),
            )
        };
        let (theirs, mine) = (run(true), run(false));
        t.check(theirs == mine && theirs.0, || {
            format!(
                "case {case}: original {:?} {:x?}; rust {:?} {:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
        drop(record);
    }
    restore(&kept, &saved);
    t.finish()
}
