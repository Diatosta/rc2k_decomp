//! `physics::sound_cue_e5` (0x45D028): when the current car's entry in `dword_641680` is
//! not -1, its sound object (`g_raceSoundObjects[g_currentCar & 7]`) gets +0x24C 0xE5,
//! +0x250 0 and +0x10 1 (a reading: a sound to play). All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller returns)
//!
//! Caller: `sub_45EDB5` (its tail).
//! Refuses (before anything) a current car past 7, or a null sound object when written.
//! Self-test: a copy of the original (the car number, both tables the test's) on test
//! objects: cars 0 .. 7, entries -1 and not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use rc2k_core::physics::listed_cars::{CUE_E5, cue_e5_object};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::sound_cue_e5", rva(0x45D028), thunk, &ORIGINAL).self_test(self_test);

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

/// The core: the car number, the listed table, the sound objects at these places.
// Converted: `rc2k_core::physics::listed_cars::{cue_e5_object, CUE_E5}`.
pub unsafe fn sound_cue_e5_at(current: u32, listed: u32, objects: u32) -> Result<Left, Refused> {
    unsafe {
        let object = cue_e5_object(
            at(current),
            |car| at(listed + 4 * car),
            |slot| at(objects + 4 * slot),
        )?;
        if let Some(object) = object {
            for (offset, v) in CUE_E5 {
                put(object + offset, v);
            }
        }
        Ok(Left::nothing())
    }
}

/// The Rust version.
pub unsafe fn sound_cue_e5() -> Result<Left, Refused> {
    let g = |a: usize| game_addr(rva(a)) as u32;
    unsafe { sound_cue_e5_at(g(0x71B3F8), g(0x641680), g(0x512A10)) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { sound_cue_e5() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<Region> = unsafe {
        let g = |a: usize| game_addr(rva(a)) as u32;
        let object = at(g(0x512A10) + 4 * (at(g(0x71B3F8)) & 7));
        if object == 0 {
            vec![]
        } else {
            vec![(object + 0x10, 4), (object + 0x24C, 8)]
        }
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The car number, 8 listed entries, 8 object pointers, two objects of 0x260 bytes.
    let mut cells = vec![0u32; 1 + 8 + 8 + 2 * 0x98];
    let base = cells.as_mut_ptr() as u32;
    let (current, listed, table) = (base, base + 4, base + 0x24);
    let objects = [base + 0x44, base + 0x44 + 0x260];
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45D028,
            len: 0x3F,
            entry: &[0x60, 0xA1, 0xF8, 0xB3, 0x71, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x71B3F8, current), (0x641680, listed), (0x512A10, table)],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    let len = cells.len() as u32;
    for car in 0..8u32 {
        for minus in [false, true] {
            let fill = || unsafe {
                for k in 0..len {
                    put(base + 4 * k, 0xA5A5_0000 | k);
                }
                for k in 0..8u32 {
                    put(table + 4 * k, objects[(k % 2) as usize]);
                }
                put(current, car);
                if minus {
                    put(listed + 4 * car, u32::MAX);
                }
            };
            fill();
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill();
            let left = unsafe { sound_cue_e5_at(current, listed, table) };
            let mine = capture(&all);
            let regs_ok = left.is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
            t.check(regs_ok && theirs == mine, || {
                format!("car {car}, entry -1 {minus}: regs_ok={regs_ok}; the memory differs")
            });
        }
    }
    drop((copy, cells));
    t.finish()
}
