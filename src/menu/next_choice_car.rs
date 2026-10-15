//! `menu::next_choice_car` (0x47CF36): the car choice's carousel turned to the next car.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/car_choice.rs`. The car choice screens' item actions (three tables). Self-test:
//! the game's carousel variables with values of the test's, 54 cases, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::menu::car_choice::{LAST_ENTRY_RVA, next_choice_car, regions};
use crate::menu::step_carousel::{COUNT_RVA, DIRECTION_RVA, FRONT_RVA};
use crate::menu::take_shown_car::SHOWN_CAR_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_choice_car", rva(0x47CF36), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { next_choice_car() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions().to_vec();
    all.extend([at(COUNT_RVA), at(LAST_ENTRY_RVA)]);
    let saved = capture(&all);
    for count in [1u32, 5] {
        for direction in [1u32, u32::MAX, 0] {
            for front in [0u32, 4, 2] {
                for shown in [0u32, 4, 2] {
                    if front == 2 && shown != 2 {
                        continue;
                    }
                    check_state(
                        &mut t,
                        &format!(
                            "count {count}, direction {direction:#x}, front {front}, shown {shown}"
                        ),
                        call_original,
                        Regs::SENTINEL,
                        &|| unsafe {
                            restore(&all, &saved);
                            game_write(COUNT_RVA, count);
                            game_write(LAST_ENTRY_RVA, 4u32);
                            game_write(DIRECTION_RVA, direction);
                            game_write(FRONT_RVA, front);
                            game_write(SHOWN_CAR_RVA, shown);
                        },
                        &regions(),
                        &rust,
                    );
                }
            }
        }
    }
    restore(&all, &saved);
    t.finish()
}
