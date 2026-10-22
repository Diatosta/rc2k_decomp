//! `race::grade_all_cars` (0x43FDFE): `race::grade_car_values` of the eight car blocks
//! (`g_carBlocks`), race init's last step.
//!
//! In:  nothing
//! Out: eax = the last block's last grade, ebx = 8, esi = the last block; the rest
//!      preserved; flags as `cmp 8, 8` leaves them (zero set, carry clear)
//!
//! See `race/car_grades.rs`. Callers: `sub_4462B9` (race init) and the code at 0x43FDDF
//! (after `sub_440A22` for each block; no function in IDA, no code reference). Shadow
//! mode: the Rust version, the grades put back, the original's, compared. Self-test: the
//! game's blocks' values set to floats round the thresholds, both versions, put back.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::car_grades::*;
use crate::race::objects::CAR_BLOCKS_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::grade_all_cars", rva(0x43FDFE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { grade_all_cars(r) };
    Ok(Left::carry(false)
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ESI, g.esi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &all_grade_regions(), &rust);
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
    let blocks: Vec<u32> = (0..8)
        .map(|n| unsafe { crate::framework::game_read(CAR_BLOCKS_RVA + 4 * n) })
        .collect();
    let regions = all_grade_regions();
    let mut kept: Vec<Region> = regions.clone();
    kept.extend(blocks.iter().map(|&b| value_region(b)));
    let saved = capture(&kept);
    for seed in [0u32, 3, 11] {
        check_state(
            &mut t,
            &format!("values by {seed}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for (n, &b) in blocks.iter().enumerate() {
                    fill_values(b, seed + n as u32);
                    std::ptr::write_bytes((b + GRADES_FROM) as *mut u8, 0x5A, GRADES_LEN);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
