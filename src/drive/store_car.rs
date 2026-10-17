//! `drive::store_car` (0x460C5D): the work copy back into the current car's block:
//! `g_car` -> `g_carBlocks[g_currentCar & 7]` (0xFE0 bytes, `rep movsd`).
//!
//! In:  nothing (`g_currentCar` 0x71B3F8)
//! Out: eax = `g_currentCar & 7`, ecx = 0, esi = 0x719FD8 + 0xFE0, edi = the block + 0xFE0
//!      (flags: the `and`'s; no caller reads them)
//!
//! See `drive/car.rs`. Callers (17): the race's frame, both halves (`sub_43703E`,
//! `sub_460B2F` under `sub_4454D2`; `sub_454253`, `sub_454265`, `sub_454289`,
//! `sub_446463`, `sub_4608BB` under `sub_4541BF`).
//! Refuses a null block (the original faults).
//! Self-test: a copy of the original whose three variables are the test's, against the
//! Rust core on the same, every slot number's low bits and some high ones; the work copy
//! and all eight blocks compared.

use crate::drive::car::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::store_car", rva(0x460C5D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust core on the variables at `p`: what it leaves in the registers.
pub unsafe fn store_car_at(p: CarPlaces) -> Result<Left, Refused> {
    unsafe {
        let (slot, block) = p.block()?;
        copy_dwords(p.work, block);
        Ok(left(slot, p.work, block))
    }
}

/// The Rust version, on the game's variables.
pub unsafe fn store_car() -> Result<Left, Refused> {
    unsafe { store_car_at(CarPlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { store_car() }
}

/// What it writes: the current car's block (none when it is null: refused).
fn regions() -> Vec<Region> {
    match unsafe { CarPlaces::game().block() } {
        Ok((_, block)) => vec![(block, CAR_LEN)],
        Err(_) => Vec::new(),
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let cars = TestCars::new();
    let p = cars.places();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x460C5D,
            len: 0x1D,
            entry: &[0x8D, 0x35, 0xD8, 0x9F, 0x71, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x71B3F8, p.current),
            (0x71B3B8, p.blocks),
            (0x719FD8, p.work),
        ],
        3,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let regions = cars.regions();
    // The work copy filled with a pattern of its own (`fill` gives it 0xAA).
    let fill = |seed: u32| {
        cars.fill(seed);
        let w = p.work as *mut u8;
        for k in 0..CAR_LEN {
            unsafe { w.add(k).write((k as u32).wrapping_mul(seed | 1) as u8) };
        }
    };
    for (n, current) in [0u32, 1, 2, 3, 4, 5, 6, 7, 8, 0xF, 0x8000_0003, u32::MAX]
        .into_iter()
        .enumerate()
    {
        let seed = n as u32 * 0x51ED_270B + 3;
        fill(seed);
        cars.set(current, 0, None);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&regions);
        fill(seed);
        let mine = unsafe { store_car_at(p) };
        let state = capture(&regions);
        let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
        t.check(regs_ok && theirs == state, || {
            format!(
                "g_currentCar {current:#x}: regs_ok={regs_ok} {mine:?}; work copy or blocks differ"
            )
        });
    }
    // A null block is refused, nothing written.
    let nulled = TestCars::new();
    let q = nulled.places();
    nulled.fill(9);
    nulled.set(0xA, 2, Some(0));
    let before = capture(&nulled.regions());
    let refused = unsafe { store_car_at(q) }.is_err();
    t.check(refused && capture(&nulled.regions()) == before, || {
        format!("a null block: refused {refused}, or something written")
    });
    t.finish()
}
