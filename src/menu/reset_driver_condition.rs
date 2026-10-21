//! `menu::reset_driver_condition` (0x47E7BC): every driver's (`g_drivers`) twelve figures
//! +0x2C .. +0x58 set to 100 (a reading: the cars' condition at an event's start).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_471F0D` (twice), the event drivers. Refuses a table without its end entry,
//! before anything. Self-test: the original and the Rust version on the game's drivers;
//! put back.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::reset_driver_condition",
    rva(0x47E7BC),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EXTRA: [usize; 0] = [];
const CASES: u32 = 2;

fn test_entry() -> Regs {
    Regs::SENTINEL
}

fn refusal_expected() -> bool {
    false
}

fn prepare(case: u32) {
    let _ = case;
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_driver_condition(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::event::driver_condition`, stored in each driver.
    driver_table()?.store_all(&rc2k_core::menu::event::driver_condition());
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    let first = game_addr(DRIVERS_RVA) as u32;
    match entries(first, DRIVER) {
        Ok(all) => vec![(first, all.len() * DRIVER as usize)],
        Err(_) => vec![(0, 0)],
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { reset_driver_condition(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let before = test_entry();
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { reset_driver_condition(&before) }.is_err();
    restore(&vars, &saved);
    if refused && !refusal_expected() {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(),
            &|| {
                restore(&vars, &saved);
                prepare(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
