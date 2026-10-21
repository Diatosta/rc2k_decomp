//! `drive::checkpoint_taken` (0x4375A5): whether checkpoint eax is no place to start a
//! car: CF when it is within 8 of the stage's start or end (`g_checkpointCount` - 8; the
//! sign of the differences) or `drive::car_near_checkpoint` finds one of the four cars
//! near it (blocks 0x70F3EC, 0x7103CC, 0x7113AC, 0x71238C). Inside `pusha` / `popa`.
//!
//! In:  eax the checkpoint
//! Out: CF; every register kept (ZF not read: `jb`)
//!
//! A `js` is in its first five bytes: `.private_copy` (its four calls relocated) with
//! `.manual_patch`.
//! Caller: `sub_43703E` (0x4371A1, 0x437284). Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original (its four blocks and the count the test's, its
//! calls to `car_near_checkpoint` kept) against the Rust core, 60 cases.

use crate::drive::car_near_checkpoint::near_at;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use rc2k_core::physics::line_places;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::checkpoint_taken", rva(0x4375A5), thunk, &ORIGINAL)
    .private_copy(0x4D)
    .relocate_calls(&[0x19, 0x26, 0x33, 0x40])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The four cars' blocks (IDA addresses).
const CARS: [usize; 4] = [0x70F3EC, 0x7103CC, 0x7113AC, 0x71238C];
const COUNT_RVA: usize = rva(0x623640); // g_checkpointCount

/// The Rust core: the four cars at `cars`, the count `count`; `r` at entry.
// Converted: `rc2k_core::physics::line_places::checkpoint_taken`.
pub unsafe fn taken_at(cars: [u32; 4], count: u32, r: &Regs) -> FlagResult {
    let result = |carry| FlagResult {
        eax: r.eax,
        ecx: None,
        ebx: None,
        carry,
        zero: None,
    };
    result(line_places::checkpoint_taken(r.eax, count, || {
        cars.iter().any(|&car| unsafe { near_at(car, r) }.carry)
    }))
}

/// The Rust version.
pub unsafe fn checkpoint_taken(r: &Regs) -> FlagResult {
    let cars = CARS.map(|a| game_addr(rva(a)) as u32);
    unsafe { taken_at(cars, game_read(COUNT_RVA), r) }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { checkpoint_taken(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const CAR: u32 = 0x260;
    let mut cells = vec![0u32; (4 * CAR / 4 + 1) as usize];
    let base = cells.as_mut_ptr() as u32;
    let cars: [u32; 4] = std::array::from_fn(|k| base + CAR * k as u32);
    let count_at = base + 4 * CAR;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4375A5,
            len: 0x4D,
            entry: &[0x60, 0x83, 0xF8, 0x08, 0x78, 0x44],
            calls: &[0x19, 0x26, 0x33, 0x40],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x70F3EC, cars[0]),
            (0x7103CC, cars[1]),
            (0x7113AC, cars[2]),
            (0x71238C, cars[3]),
            (0x623640, count_at),
        ],
        5,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut case = 0u32;
    for count in [10u32, 40, 0x200] {
        for at in [0u32, 7, 8, 9, 20, 31, 32, 33, 0x1F7, 0x1F8, 0x8000_0000, 15] {
            case += 1;
            for (k, &car) in cars.iter().enumerate() {
                let s = case.wrapping_mul(31).wrapping_add(k as u32 * 7);
                unsafe {
                    ((car + 0x25C) as *mut u32).write(s % 3);
                    ((car + 0x98) as *mut u32).write(at.wrapping_add(s % 11).wrapping_sub(5));
                }
            }
            unsafe { (count_at as *mut u32).write(count) };
            let before = Regs {
                eax: at,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let mine = unsafe { taken_at(cars, count, &before) };
            t.check(mine.matches(&before, &ro), || {
                format!(
                    "count {count:#x}, checkpoint {at:#x}: original carry {}; rust {mine:?}",
                    ro.carry()
                )
            });
        }
    }
    drop((copy, cells));
    t.finish()
}
