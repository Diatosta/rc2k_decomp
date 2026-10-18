//! `race::store_car_b` (0x454265): `g_currentCar` the view car `dword_63DBAC`, then
//! `drive::store_car` (the work copy into the car's block).
//!
//! In:  nothing
//! Out: as `drive::store_car` leaves the registers
//!
//! See `race/frame.rs`. Callers under `sub_4541BF` (the drawing). Refuses a null block (the
//! original faults), `g_currentCar` put back. Shadow mode: the Rust version, the block and
//! `g_currentCar` put back, the original's, compared. Self-test: the game's blocks, each
//! filled with 0x5A before each run, view cars 0, 1, 5, 7 and 9; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::store_car_b", rva(0x454265), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { store_view_car(1) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &store_regions(1), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved_view = capture(&[at(VIEW_CARS[1])]);
    for car in [0u32, 1, 5, 7, 9] {
        unsafe { game_write(VIEW_CARS[1], car) };
        let regions = store_regions(1);
        if regions.len() < 2 {
            restore(&[at(VIEW_CARS[1])], &saved_view);
            return t.skip("a car block is null (no race loaded)");
        }
        let saved = capture(&regions);
        check_state(
            &mut t,
            &format!("view car {car}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(VIEW_CARS[1], car);
                std::ptr::write_bytes(regions[1].0 as *mut u8, 0x5A, regions[1].1);
            },
            &regions,
            &rust,
        );
        restore(&regions, &saved);
    }
    restore(&[at(VIEW_CARS[1])], &saved_view);
    t.finish()
}
