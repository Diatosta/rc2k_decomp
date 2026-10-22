//! `menu::link_player_names` (0x47059E): hangs each player's name on that player's car.
//!
//! In:  nothing
//! Out: every register preserved
//! For each of the four players (`g_cars`): its +0x18 = the address of the player's
//! name in `g_playerNames`, and the name's 0x20 bytes copied to its +0x124.
//! Callers: `saves::load_savegame` (twice) and the main menu's set-up `sub_47C63F`.
//! Refuses a missing player.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::player_names::{
    CAR_NAME, CAR_NAME_POINTER, NAME_LEN, cars, name, regions, write_test_names,
};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::link_player_names", rva(0x47059E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn link_player_names() -> Result<(), BadInput> {
    unsafe {
        for (player, car) in cars()?.into_iter().enumerate() {
            ((car as usize + CAR_NAME_POINTER) as *mut u32).write_unaligned(name(player) as u32);
            std::ptr::copy(name(player), (car as usize + CAR_NAME) as *mut u8, NAME_LEN);
        }
        Ok(())
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { link_player_names() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    match unsafe { cars() } {
        Ok(cars) => dispatch_state(&HOOK, r, call_original, &regions(&cars), &rust),
        Err(e) => {
            HOOK.count_call();
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
        }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { cars() }?;
    crate::service::harness::state_entry(r, &rust)
}

// ---- self-test: names of the test's, on the game's cars, all put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(cars) = (unsafe { cars() }) else {
        return t.skip("a player is missing");
    };
    let regions = regions(&cars);
    let saved = capture(&regions);
    check_state(
        &mut t,
        "four names",
        call_original,
        Regs::SENTINEL,
        &|| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            write_test_names();
        },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
