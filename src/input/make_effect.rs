//! `input::make_effect` (0x40992F): one of a device record's six force-feedback effects
//! made: an object it had given back (`input::release_effect`); its `DIEFFECT` (+0x34,
//! 0x34 bytes: cartesian, by object offsets, for ever, gain 10000, no trigger, one axis, x
//! (+0x80; +0x84 4), direction 0 (+0x88), no envelope (one of 0x14 bytes at +0x90, all 0,
//! not linked)) and its type-specific parameters (+0x68) by its kind (+4): 1 a constant
//! force of 10000; 2 a ramp from 10000 to 10000; 3 a periodic one (magnitude 10000, offset
//! 0, phase 0, period 50000); 4 a condition (offset 0, both coefficients and saturations
//! 10000, dead band 0); others none (no size, no pointer); then its function (+0) called
//! with eax 1; and with an effect found for it (+0xC 1, `input::effect_found`) its object
//! made (`CreateEffect` of the record's device (+4) with the GUID found (+0x10), into
//! +0x30; its result not looked at).
//!
//! In:  esi = the device's record, edi = the effect, ecx = its number (for its function)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::begin_force` (each of the six). The effect's function through `part`
//! (its address; eax 1, the other registers as here; in the game through the game, one of
//! `input::effect_N_change`), the methods the objects' own. Shadow mode runs the original
//! alone (it calls DirectInput). Self-test: the original and the Rust version on
//! DirectInput objects made in Rust (`input::di_harness`), the effect's function a noting
//! stand-in: each kind and one past them, found or not, an object before or not, the wheel
//! (force feedback) and the joystick (none: `CreateEffect` fails).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva, shadow};
use crate::input::devices::{at, put};
use crate::input::di_harness::{compare, method, regs_text, slot};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::make_effect", rva(0x40992F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The effect's `DIEFFECT` and parameters, as the original writes them (in its order).
pub fn fill_parameters(e: u32) {
    let w = |offset: u32, v: u32| put(e, offset, v);
    w(0x34, 0x34);
    w(0x38, 0x12);
    w(0x3C, u32::MAX);
    w(0x40, 0);
    w(0x44, 10000);
    w(0x48, u32::MAX);
    w(0x4C, 0);
    w(0x50, 1);
    w(0x54, e.wrapping_add(0x80));
    w(0x80, 0);
    w(0x84, 4);
    w(0x58, e.wrapping_add(0x88));
    w(0x88, 0);
    w(0x5C, 0);
    w(0x60, 0);
    w(0x64, 0);
    for offset in [0x90, 0x94, 0x98, 0x9C, 0xA0] {
        w(offset, if offset == 0x90 { 0x14 } else { 0 });
    }
    let params: &[u32] = match at(e, 4) {
        1 => &[10000],
        2 => &[10000, 10000],
        3 => &[10000, 0, 0, 50000],
        4 => &[0, 10000, 10000, 10000, 10000, 0],
        _ => return,
    };
    w(0x60, 4 * params.len() as u32);
    w(0x64, e.wrapping_add(0x68));
    for (n, &v) in params.iter().enumerate() {
        w(0x68 + 4 * n as u32, v);
    }
}

/// A call by place: 0 the effect's function (its address in `at`).
pub type Part<'a> = &'a mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header).
pub fn make_effect(entry: &Regs, part: Part) -> Result<(), Refused> {
    let e = entry.edi;
    if at(e, 0x30) != 0 {
        crate::input::effect_objects::release_effect(e);
    }
    fill_parameters(e);
    let mut r = Regs { eax: 1, ..*entry };
    part(at(e, 0) as usize, &mut r)?;
    if at(e, 0xC) == 1 {
        method(
            at(entry.esi, 4),
            slot::CREATE_EFFECT,
            &[
                e.wrapping_add(0x10),
                e.wrapping_add(0x34),
                e.wrapping_add(0x30),
                0,
            ],
        );
    }
    Ok(())
}

/// In the game: the effect's function through the game.
pub fn game_part(function: usize, r: &mut Regs) -> Result<(), Refused> {
    unsafe { call_game(r, function) };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = make_effect(r, &mut game_part) {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

static NOTED: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn noting(place: u32, r: &mut Regs) {
    NOTED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(format!("{place}: {}", regs_text(r)));
}

fn take_noted() -> String {
    std::mem::take(&mut *NOTED.lock().unwrap_or_else(|e| e.into_inner())).join("; ")
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let world = crate::input::di_objects::TestWorld::begin(
        crate::input::di_objects::test_devices(),
        Default::default(),
    );
    crate::install::stand_ins::set_handler(noting);
    let stand_in = crate::install::stand_ins::stand_in(0) as u32;
    let mut record = crate::input::devices::test_record(7);
    let rec = record.as_mut_ptr() as u32;
    let start = record.clone();
    for device in [1usize, 0] {
        for kind in 1..=5u32 {
            for found in [1u32, 0] {
                for before in [false, true] {
                    // the third effect: ecx 2
                    let e = rec + 0x3C + 2 * 0xA4;
                    let prepare = || {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                start.as_ptr(),
                                rec as *mut u8,
                                start.len(),
                            )
                        };
                        crate::input::effect_objects::test_effect(e, before);
                        put(e, 0, stand_in);
                        put(e, 4, kind);
                        put(e, 0xC, found);
                        put(rec, 4, crate::input::di_objects::test_device(device));
                        take_noted();
                    };
                    let entry = Regs {
                        ecx: 2,
                        esi: rec,
                        edi: e,
                        ..Regs::SENTINEL
                    };
                    compare(
                        &mut t,
                        &format!(
                            "device {device}, kind {kind}, found {found}, an object before {before}"
                        ),
                        &world,
                        &[(rec, crate::input::devices::RECORD_LEN)],
                        &prepare,
                        &mut || {
                            let mut r = entry;
                            unsafe { call_original(&mut r) };
                            format!("{}; noted {}", regs_text(&r), take_noted())
                        },
                        &mut || {
                            let mut part = |_: usize, r: &mut Regs| {
                                crate::install::stand_ins::act(0, r);
                                Ok(())
                            };
                            let done = make_effect(&entry, &mut part);
                            format!("{} {done:?}; noted {}", regs_text(&entry), take_noted())
                                .replace(" Ok(())", "")
                        },
                    );
                }
            }
        }
    }
    drop(world);
    drop(record);
    t.finish()
}
