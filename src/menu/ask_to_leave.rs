//! `menu::ask_to_leave` (0x47E8BB): `dword_755569` 1: the screen loop asks whether to leave
//! (the review menus' "back to the main menu", a reading).
//!
//! In:  nothing
//! Out: every register kept
//!
//! An item action in six menu tables (the review menus). Self-test: the original and the
//! Rust version on the variable, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::ask_to_leave", rva(0x47E8BB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

use crate::menu::leaving::ASK_RVA;

/// The Rust version (see the header).
pub fn ask_to_leave() {
    // Converted: `rc2k_core::menu::leaving::Leaving::ask_to_leave`.
    crate::menu::leaving::with_leaving(|l| l.ask_to_leave());
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    ask_to_leave();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(ASK_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(ASK_RVA)];
    let saved = capture(&regions);
    for before in [0u32, 1, 0xFFFF_FFFF] {
        check_state(
            &mut t,
            &format!("before {before:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(ASK_RVA, before) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
