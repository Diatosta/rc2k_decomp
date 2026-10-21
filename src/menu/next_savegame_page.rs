//! `menu::next_savegame_page` (0x46D407): the next page of the saved games' list: adds
//! 1 to `dword_75523C`, and above 9 goes back to 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The comparison is signed. In two menu tables (0x7518D0, 0x751D28): the load and
//! save screens' page buttons. The twin of `service::next_setup_page`.
//! Self-test: eight pages.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::name_lists::SAVEGAME_PAGE_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_savegame_page", rva(0x46D407), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub fn next_savegame_page() {
    unsafe {
        // Converted: `rc2k_core::menu::steps::savegame_page`.
        let page = game_read::<u32>(SAVEGAME_PAGE_RVA);
        game_write(
            SAVEGAME_PAGE_RVA,
            rc2k_core::menu::steps::savegame_page(page, true),
        );
    }
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [at(SAVEGAME_PAGE_RVA)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    next_savegame_page();
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
    for page in [-5i32, -1, 0, 1, 8, 9, 10, i32::MAX] {
        check_state(
            &mut t,
            &format!("page={page}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(SAVEGAME_PAGE_RVA, page) },
            &regions(),
            &rust,
        );
    }
    restore(&regions(), &saved);
    t.finish()
}
