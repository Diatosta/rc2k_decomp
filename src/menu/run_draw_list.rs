//! `menu::run_draw_list` (0x4681ED): `run_list` on the current screen's draw list, the one
//! `dword_7554F8` points at.
//!
//! In:  nothing
//! Out: every register preserved
//! Three callers, the menu's frame step among them. Shadow mode runs the original alone, as
//! for `run_list`. Refuses a list without an end in its first 256 entries.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::run_list::{Call, check, in_game, run_list, test_lists};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::run_draw_list", rva(0x4681ED), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const DRAW_LIST_RVA: usize = rva(0x7554F8); // dword_7554F8

// ---- Rust version ----

pub unsafe fn run_draw_list(entry_regs: &Regs, call: Call) -> Result<(), BadInput> {
    unsafe { run_list(game_read(DRAW_LIST_RVA), entry_regs, call) }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let entry_regs = *r;
    if shadow() || unsafe { run_draw_list(&entry_regs, &mut in_game) }.is_err() {
        unsafe { call_original(r) };
    }
}

// ---- self-test: the list pointer is aimed at lists of stand-ins, then put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved: u32 = unsafe { game_read(DRAW_LIST_RVA) };
    for (n, list) in test_lists().iter().enumerate() {
        unsafe { game_write(DRAW_LIST_RVA, list.as_ptr() as u32) };
        check(
            &mut t,
            &format!("list {n}"),
            Regs::SENTINEL,
            &|r| unsafe { call_original(r) },
            &|r| unsafe { run_draw_list(r, &mut in_game) },
        );
    }
    unsafe { game_write(DRAW_LIST_RVA, saved) };
    t.finish()
}
