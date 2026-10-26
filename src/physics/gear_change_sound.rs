//! `physics::gear_change_sound` (0x45C551): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`, in edi) told of `g_car` +0x238: at 1 its
//! +0x180 and +0x10 set to 1; at -1 its +0x198 and +0x10 set to 1. A reading: a gear
//! change up or down heard.
//!
//! In:  nothing
//! Out: edi the sound object; the rest kept; no flags read (the caller returns)
//!
//! Caller: `sub_43DC72` (its tail).
//! Refuses (before anything) a null sound object when it would be written (the original
//! writes near address 0).
//! Self-test: a copy of the original (the car number, the object table and the word the
//! test's) on test objects: words 1, -1, 0, 2, cars 0 .. 9.

use crate::errors::{BadInput, Refused};
use crate::framework::{EDI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, state_entry};
use rc2k_core::physics::gearbox::{GearChange, gear_change};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::gear_change_sound",
    rva(0x45C551),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust version on the car number, the object table and the word at these places.
// Converted: `rc2k_core::physics::gearbox::gear_change`.
pub unsafe fn gear_change_sound_at(current: u32, table: u32, word: u32) -> Result<Left, Refused> {
    unsafe {
        let object = at(table + 4 * (at(current) & 7));
        if let Some(change) = gear_change(at(word)) {
            if object == 0 {
                return Err(BadInput("a null sound object").into());
            }
            let flag = match change {
                GearChange::Up => 0x180,
                GearChange::Down => 0x198,
            };
            put(object + flag, 1);
            put(object + 0x10, 1);
        }
        Ok(Left::nothing().with(EDI, object))
    }
}

/// The Rust version.
pub unsafe fn gear_change_sound() -> Result<Left, Refused> {
    let g = |a: usize| game_addr(rva(a)) as u32;
    unsafe { gear_change_sound_at(g(0x71B3F8), g(0x512A10), g(0x71A210)) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { gear_change_sound() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<Region> = unsafe {
        let g = |a: usize| game_addr(rva(a)) as u32;
        let object = at(g(0x512A10) + 4 * (at(g(0x71B3F8)) & 7));
        if object == 0 {
            vec![]
        } else {
            vec![(object + 0x10, 4), (object + 0x180, 4), (object + 0x198, 4)]
        }
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The car number, the word, 8 object pointers, two objects of 0x1A0 bytes.
    let mut cells = vec![0u32; 2 + 8 + 2 * 0x68];
    let base = cells.as_mut_ptr() as u32;
    let (current, word, table) = (base, base + 4, base + 8);
    let objects = [base + 0x28, base + 0x28 + 0x1A0];
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45C551,
            len: 0x45,
            entry: &[0x8B, 0x3D, 0xF8, 0xB3, 0x71, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x71B3F8, current), (0x512A10, table), (0x71A210, word)],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    for car in 0..10u32 {
        for w in [1u32, u32::MAX, 0, 2] {
            let fill = || unsafe {
                for k in 0..(cells.len() as u32) {
                    put(base + 4 * k, 0xA5A5_0000 | k);
                }
                for k in 0..8u32 {
                    put(table + 4 * k, objects[(k % 2) as usize]);
                }
                put(current, car);
                put(word, w);
            };
            fill();
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill();
            let left = unsafe { gear_change_sound_at(current, table, word) };
            let mine = capture(&all);
            let regs_ok = left.is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
            t.check(regs_ok && theirs == mine, || {
                format!("car {car}, word {w:#x}: regs_ok={regs_ok}; the memory differs")
            });
        }
    }
    unsafe {
        put(table, 0);
        put(current, 0);
        put(word, 1);
    }
    let refused = unsafe { gear_change_sound_at(current, table, word) }.is_err();
    t.check(refused, || "a null object not refused".into());
    drop((copy, cells));
    t.finish()
}
