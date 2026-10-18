//! `race::load_first_car` (0x460C7A): `g_currentCar` 0, `drive::load_car` (see
//! `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as `drive::load_car` leaves them
//!
//! Callers: 0x429474, 0x42948D, `sub_45429B` (0x45472E), 0x459401. Refuses (before
//! anything) car 0 without its block. Shadow mode: by state. Self-test: `g_currentCar` 3
//! and 0 before, with a block (skipped without); all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame_views::load_first_car;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::load_first_car", rva(0x460C7A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_first_car() }
}

fn regions() -> Vec<Region> {
    vec![(game_addr(rva(0x719FD8)) as u32, 0xFE0), at(rva(0x71B3F8))]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(rva(0x71B3B8)) } == 0 {
        return t.skip("car 0 has no block");
    }
    let regions = regions();
    let saved = capture(&regions);
    for before in [3u32, 0] {
        check_state(
            &mut t,
            &format!("current car {before}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved);
                game_write(rva(0x71B3F8), before);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
