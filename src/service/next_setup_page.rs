//! `service::next_setup_page` (0x46F191): shows the next page of the setup slot list: adds 1 to
//! `dword_755244`, and from 3 on goes back to 0.
//!
//! In:  nothing
//! Out: every register preserved
//! The comparison is signed, so a negative page just goes up by one. In two menu tables.

use crate::errors::Refused;
use crate::framework::game_read;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::service::load_setup_slot::SLOT_PAGE_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::next_setup_page", rva(0x46F191), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub fn next_setup_page() {
    unsafe {
        let next = game_read::<i32>(SLOT_PAGE_RVA).wrapping_add(1);
        game_write(SLOT_PAGE_RVA, if next < 3 { next } else { 0 });
    }
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [at(SLOT_PAGE_RVA)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    next_setup_page();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = capture(&regions());
    for page in [-5i32, -1, 0, 1, 2, 3, 100, i32::MAX] {
        check_state(
            &mut t,
            &format!("page={page}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(SLOT_PAGE_RVA, page) },
            &regions(),
            &rust,
        );
    }
    restore(&regions(), &saved);
    t.finish()
}
