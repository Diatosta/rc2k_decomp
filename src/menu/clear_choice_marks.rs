//! `menu::clear_choice_marks` (0x490346): `dword_75AAC8` .. `dword_75AAD4` set to -1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Four dwords, one a player by the look of it (their meaning is not read). In two entry
//! lists (0x759406, 0x759412: the car choice's). Self-test: the four filled with 0xAA, put
//! back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::clear_choice_marks", rva(0x490346), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MARKS_RVA: usize = rva(0x75AAC8); // dword_75AAC8 .. dword_75AAD4

fn regions() -> [Region; 1] {
    [(game_addr(MARKS_RVA) as u32, 16)]
}

pub fn clear_choice_marks() {
    unsafe { std::ptr::write_bytes(game_addr(MARKS_RVA) as *mut u8, 0xFF, 16) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    clear_choice_marks();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "filled with 0xAA",
        call_original,
        Regs::SENTINEL,
        &|| unsafe { std::ptr::write_bytes(regions[0].0 as *mut u8, 0xAA, 16) },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
