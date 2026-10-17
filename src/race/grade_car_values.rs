//! `race::grade_car_values` (0x43FE24): car block esi's 34 values graded: each grade the
//! number of its thresholds (50, 75, 90, 100 or fewer; see `GRADES`) below the value,
//! tried in order by `fild` / `fcomp` / `fnstsw` (C0: a NaN passes them all).
//!
//! In:  esi = the car block
//! Out: eax = the last grade; the rest preserved; `dword_62400C` the last threshold tried
//!
//! See `race/car_grades.rs`. One caller, `race::grade_all_cars` (its flags are not read
//! there: `pop` / `inc` / `cmp` follow). Shadow mode: the Rust version, the grades and
//! `dword_62400C` put back, the original's, compared. Self-test: the game's first block's
//! values set to floats round the thresholds (NaNs, infinities, a denormal, negatives),
//! sixteen ways, both versions, put back.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::car_grades::*;
use crate::race::objects::CAR_BLOCKS_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::grade_car_values", rva(0x43FE24), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { grade_car_values(r) };
    Ok(Left::nothing().with(EAX, g.eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &grade_regions(r.esi), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let b: u32 = unsafe { crate::framework::game_read(CAR_BLOCKS_RVA) };
    let regions = grade_regions(b);
    let mut kept = regions.clone();
    kept.push(value_region(b));
    let saved = capture(&kept);
    for seed in 0..16u32 {
        check_state(
            &mut t,
            &format!("values by {seed}"),
            call_original,
            Regs {
                esi: b,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                fill_values(b, seed);
                std::ptr::write_bytes((b + GRADES_FROM) as *mut u8, 0x5A, GRADES_LEN);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
