//! `race::load_car_c` (0x454277): `g_currentCar` the view car `dword_63DBA0`, then `drive::load_car`
//! (the car's block into the work copy).
//!
//! In:  nothing
//! Out: as `drive::load_car` leaves the registers
//!
//! See `race/frame.rs`. one caller, `sub_45B4E6`. Shadow mode: the Rust version, the work copy and
//! `g_currentCar` put back, the original's, compared. Self-test: the game's blocks, view
//! cars 0 .. 7, all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::load_car_c", rva(0x454277), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_view_car(2) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &load_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = load_regions();
    let mut kept = regions.clone();
    kept.push(at(VIEW_CARS[2]));
    let saved = capture(&kept);
    for car in [0u32, 1, 5, 7, 9] {
        check_state(
            &mut t,
            &format!("view car {car}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(VIEW_CARS[2], car) },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
