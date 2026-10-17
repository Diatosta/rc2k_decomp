//! `drive::load_car` (0x460C40): the current car's block into the work copy:
//! `g_carBlocks[g_currentCar & 7]` -> `g_car` (0xFE0 bytes, `rep movsd`).
//!
//! In:  nothing (`g_currentCar` 0x71B3F8)
//! Out: eax = `g_currentCar & 7`, ecx = 0, esi = the block + 0xFE0, edi = 0x719FD8 + 0xFE0
//!      (flags: the `and`'s; no caller reads them)
//!
//! See `drive/car.rs`. Callers (18): the race's frame, both halves (`sub_43703E`,
//! `sub_460B2F` under `sub_4454D2`; `sub_45422F`, `sub_454241`, `sub_454277`,
//! `sub_446463`, `sub_45754C`, `sub_4608BB`, `sub_460C7A` under `sub_4541BF`).
//! Refuses a null block (the original faults).
//! Self-test: a copy of the original whose three variables are the test's (`g_car` is used
//! all through a frame), against the Rust core on the same, every slot number's low bits
//! and some high ones.

use crate::drive::car::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::load_car", rva(0x460C40), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust core on the variables at `p`: what it leaves in the registers.
pub unsafe fn load_car_at(p: CarPlaces) -> Result<Left, Refused> {
    unsafe {
        let (slot, block) = p.block()?;
        copy_dwords(block, p.work);
        Ok(left(slot, block, p.work))
    }
}

/// The Rust version, on the game's variables.
pub unsafe fn load_car() -> Result<Left, Refused> {
    unsafe { load_car_at(CarPlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_car() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let work = (crate::framework::game_addr(CAR_RVA) as u32, CAR_LEN);
    dispatch_state(&HOOK, r, call_original, &[work], &rust);
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
            address: 0x460C40,
            len: 0x1D,
            entry: &[0xA1, 0xF8, 0xB3, 0x71, 0x00],
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
    for (n, current) in [0u32, 1, 2, 3, 4, 5, 6, 7, 8, 0xF, 0x8000_0003, u32::MAX]
        .into_iter()
        .enumerate()
    {
        let seed = n as u32 * 0x51ED_270B + 1;
        cars.fill(seed);
        cars.set(current, 0, None);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&regions);
        cars.fill(seed);
        let mine = unsafe { load_car_at(p) };
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
    nulled.fill(7);
    nulled.set(5, 5, Some(0));
    let before = capture(&nulled.regions());
    let refused = unsafe { load_car_at(q) }.is_err();
    t.check(refused && capture(&nulled.regions()) == before, || {
        format!("a null block: refused {refused}, or something written")
    });
    t.finish()
}
