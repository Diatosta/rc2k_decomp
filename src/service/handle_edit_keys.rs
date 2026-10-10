//! `service::handle_edit_keys` (0x46B890): while an item of the service area is being edited
//! (`dword_75AA9C == 1`), passes the keys to the edit page in `dword_75AAA0`: 1 gear ratio,
//! 2 tyre, 3 steering sensitivity, 4 brake balance, 5 ride height, 6 suspension stiffness.
//!
//! In:  nothing
//! Out: every register preserved
//! Any other page number is fatal error 0x80709A9A, "Incorrect Locked Box", in the original.
//! It is not called directly: its address is in the service area's list at 0x758D6E.

use crate::errors::FatalError;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::framework::{game_read, game_write};
use crate::screens::harness::Left;
use crate::service::edit_brake_balance::edit_brake_balance;
use crate::service::edit_gear_ratio::edit_gear_ratio;
use crate::service::edit_ride_height::edit_ride_height;
use crate::service::edit_steering_sensitivity::edit_steering_sensitivity;
use crate::service::edit_suspension_stiffness::edit_suspension_stiffness;
use crate::service::edit_tyre::edit_tyre;
use crate::service::harness::dispatch_keyed_state;
use crate::service::harness::{Region, check_state_with};
use crate::service::pages::{
    BRAKE_BALANCE, BUDGETS, GEAR_RATIO, PageTest, RIDE_HEIGHT, SCENARIOS, STEERING_SENSITIVITY,
    SUSPENSION_STIFFNESS, TEST_KEYS, TYRE, observe_clock, page_regions, pages_unavailable,
    prepare_change, prepare_simple, with_key,
};
use crate::service::state::{EDIT_MODE_RVA, EDIT_PAGE_RVA, edit_mode};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::handle_edit_keys", rva(0x46B890), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn handle_edit_keys() -> Result<(), Refused> {
    if !edit_mode() {
        return Ok(());
    }
    unsafe {
        match game_read::<u32>(EDIT_PAGE_RVA) {
            1 => edit_gear_ratio(false),
            2 => edit_tyre(false),
            3 => edit_steering_sensitivity(),
            4 => edit_brake_balance(),
            5 => edit_ride_height(false),
            6 => edit_suspension_stiffness(false),
            _ => Err(FatalError {
                code: 0x8070_9A9A,
                message: "Incorrect Locked Box",
            }
            .into()),
        }
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    page_regions(
        &[
            GEAR_RATIO.change.chosen_rva,
            TYRE.change.chosen_rva,
            RIDE_HEIGHT.change.chosen_rva,
            SUSPENSION_STIFFNESS.change.chosen_rva,
        ],
        &[STEERING_SENSITIVITY.car_offset, BRAKE_BALANCE.car_offset],
    )
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { handle_edit_keys() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: every page with every key; never a page number outside 1 .. 6 ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = pages_unavailable() {
        return t.skip(why);
    }
    let Some(test) = PageTest::begin() else {
        return t.finish();
    };
    let regions = with_key(regions());
    let changed = SCENARIOS[1].1;
    for page in 1u32..=6 {
        for key in TEST_KEYS {
            for mode in [0u32, 1, 2] {
                for budget in BUDGETS {
                    let prepare = || {
                        match page {
                            1 => prepare_change(GEAR_RATIO, key, changed, 0, budget),
                            2 => prepare_change(TYRE, key, changed, 0, budget),
                            3 => prepare_simple(STEERING_SENSITIVITY, key, 50),
                            4 => prepare_simple(BRAKE_BALANCE, key, 50),
                            5 => prepare_change(RIDE_HEIGHT, key, changed, 0, budget),
                            _ => prepare_change(SUSPENSION_STIFFNESS, key, changed, 0, budget),
                        }
                        unsafe {
                            game_write(EDIT_PAGE_RVA, page);
                            game_write(EDIT_MODE_RVA, mode);
                        }
                    };
                    check_state_with(
                        &mut t,
                        &format!("page={page} key={key:#x} mode={mode} budget={budget}"),
                        call_original,
                        Regs::SENTINEL,
                        &prepare,
                        &regions,
                        &rust,
                        &observe_clock,
                    );
                }
            }
        }
    }
    test.end();
    t.finish()
}
