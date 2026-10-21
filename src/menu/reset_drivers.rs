//! `menu::reset_drivers` (0x470399): every driver (`g_drivers`, 0x94 each) given +0x24 1,
//! +0xC 0 and +0x10 0 (a reading: in the event, no points, no place).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers and `sub_4A9C9B`. Refuses a table without its end entry,
//! before anything. Self-test: the original and the Rust version on the game's drivers;
//! put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::reset_drivers", rva(0x470399), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_drivers(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::event::DRIVER_RESET`, stored in each driver.
    driver_table()?.store_all(&rc2k_core::menu::event::DRIVER_RESET);
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let first = game_addr(DRIVERS_RVA) as u32;
    match entries(first, DRIVER) {
        Ok(all) => vec![(first, all.len() * DRIVER as usize)],
        Err(_) => vec![],
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { reset_drivers(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    if regions.is_empty() {
        return t.skip("the drivers' table without its end");
    }
    let saved = capture(&regions);
    check_state(
        &mut t,
        "the game's drivers",
        call_original,
        Regs::SENTINEL,
        &|| restore(&regions, &saved),
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
