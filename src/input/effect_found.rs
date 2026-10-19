//! `input::effect_found` (0x40953B): DirectInput's `EnumEffects` callback: each of the
//! device record's six effects (from +0x3C, 0xA4 bytes) whose kind (+4) is the found
//! effect's type (`DIEFFECTINFOA` +0x14, its low byte) takes it, unless it already has
//! one (+0xC 1) that is the GUID it wants (+8 points at it): +0xC 1, +0x20 .. +0x28 the
//! info's type, static and dynamic parameters, +0x10 its GUID. So each effect ends with
//! its wanted GUID when the device has it, else with the last of its type listed. Always
//! goes on (eax 1).
//!
//! In:  (stdcall) the `DIEFFECTINFOA`, the context (the device's record)
//! Out: eax = 1; ebx, esi, edi, ebp kept; `ret 8`
//!
//! Caller: DirectInput, from `input::begin_force`. Shadow mode runs the original alone.
//! Self-test: the original and the Rust version called with the eleven standard effects in
//! turn, as `EnumEffects` lists them, on a copy of the first device's record whose effects
//! have found nothing, their wanted GUID, or another one.

use crate::framework::{RawHook, SelfTest, game_addr, rva};
use crate::input::devices::{RECORD_LEN, RECORDS, at, put};
use crate::input::di_harness::{EFFECT_LEN, EFFECTS_AT, call_stdcall2, compare, dispatch_callback};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn thunk(info: u32, record: u32) -> u32 {
    dispatch_callback(
        &HOOK,
        ORIGINAL.load(Ordering::SeqCst),
        info,
        record,
        effect_found,
    )
}

type Thunk = unsafe extern "system" fn(u32, u32) -> u32;

pub static HOOK: RawHook = RawHook::new(
    "input::effect_found",
    rva(0x40953B),
    unsafe { std::mem::transmute::<Thunk, unsafe extern "C" fn()>(thunk) },
    &ORIGINAL,
)
.self_test(self_test);

/// The Rust version: eax.
pub fn effect_found(info: u32, record: u32) -> u32 {
    let kind = at(info, 0x14) & 0xFF;
    for n in 0..6 {
        let e = record.wrapping_add(EFFECTS_AT + n * EFFECT_LEN);
        if kind != at(e, 4) {
            continue;
        }
        if at(e, 0xC) == 1 && crate::system::same_guid::same_guid(e.wrapping_add(0x10), at(e, 8)) {
            continue;
        }
        put(e, 0xC, 1);
        put(e, 0x20, at(info, 0x14));
        put(e, 0x24, at(info, 0x18));
        put(e, 0x28, at(info, 0x1C));
        unsafe { std::ptr::copy((info + 4) as *const u8, e.wrapping_add(0x10) as *mut u8, 16) };
    }
    1
}

/// The eleven standard effects' `DIEFFECTINFOA`s, as `EnumEffects` gives them.
pub fn test_infos() -> Vec<Vec<u8>> {
    (0..11u32)
        .map(|n| {
            let mut b = vec![0u8; 0x124];
            b[0..4].copy_from_slice(&0x124u32.to_le_bytes());
            let g = crate::input::di_objects::guid(
                0x1354_1C20 + n,
                0x8E33,
                0x11D0,
                [0x9A, 0xD0, 0x00, 0xA0, 0xC9, 0xA0, 0x6E, 0x35],
            );
            b[4..0x14].copy_from_slice(&g);
            let family: u32 = match n {
                0 => 1,
                1 => 2,
                2..=6 => 3,
                _ => 4,
            };
            // the type with a flag above its low byte, and parameters of the test's
            b[0x14..0x18].copy_from_slice(&(family | 0x200).to_le_bytes());
            b[0x18..0x1C].copy_from_slice(&(0x100 + n).to_le_bytes());
            b[0x1C..0x20].copy_from_slice(&(0x200 + n).to_le_bytes());
            b
        })
        .collect()
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let world = crate::input::di_objects::TestWorld::begin(Vec::new(), Default::default());
    let infos = test_infos();
    let real =
        unsafe { std::slice::from_raw_parts(game_addr(RECORDS[0]) as *const u8, RECORD_LEN) };
    let start = real.to_vec();
    let mut record = start.clone();
    let rec = record.as_mut_ptr() as u32;
    let original = ORIGINAL.load(Ordering::SeqCst);
    // what each effect has found: 0 nothing, 1 its wanted GUID, 2 another (the friction's)
    for found in [
        [0u32; 6],
        [1; 6],
        [2; 6],
        [0, 1, 2, 0, 1, 2],
        [2, 0, 1, 2, 0, 1],
    ] {
        let prepare = || {
            unsafe { std::ptr::copy_nonoverlapping(start.as_ptr(), rec as *mut u8, RECORD_LEN) };
            for (n, &f) in found.iter().enumerate() {
                let e = rec + EFFECTS_AT + n as u32 * EFFECT_LEN;
                put(e, 0xC, (f != 0) as u32);
                let wanted = at(e, 8);
                let from = if f == 1 {
                    wanted
                } else {
                    infos[10].as_ptr() as u32 + 4
                };
                if f != 0 {
                    unsafe { std::ptr::copy(from as *const u8, (e + 0x10) as *mut u8, 16) };
                }
            }
        };
        let run = |f: &dyn Fn(u32, u32) -> u32| {
            infos
                .iter()
                .map(|i| format!("{}", f(i.as_ptr() as u32, rec)))
                .collect::<Vec<_>>()
                .join(",")
        };
        compare(
            &mut t,
            &format!("found {found:?}"),
            &world,
            &[(rec, RECORD_LEN)],
            &prepare,
            &mut || run(&|a, b| unsafe { call_stdcall2(original, a, b) }),
            &mut || run(&effect_found),
        );
    }
    drop(world);
    drop(record);
    t.finish()
}
