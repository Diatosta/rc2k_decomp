//! `menu::move_highlight_right` (0x474766): moves the highlight along the items' link at +0x30, on
//! key 0x69.
//!
//! In:  nothing (edi is the menu at its one call, and is not used)
//! Out: every register preserved
//! One step; while `dword_7554EC` is 1 it first turns the counter `dword_75AA7C` forwards.
//! The rules, the click and the item's callback (+0x40) are in `menu/navigation.rs`.
//! One caller, `menu::handle_keys`. Shadow mode: the original runs for real, then the Rust
//! version without the click and the callback, and the two are compared unless a callback
//! was left out. Self-test: a redirected copy whose click is a stand-in.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::navigation::{
    Direction, Outside, check_move, dispatch_outside, move_highlight, move_regions,
};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::move_highlight_right",
    rva(0x474766),
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

// ---- Rust version: `navigation::move_highlight` with `Direction::Right` ----

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_outside(
        &HOOK,
        r,
        call_original,
        &move_regions(),
        &|r: &Regs, out: &mut dyn Outside| {
            unsafe { move_highlight(Direction::Right, r, out) }?;
            Ok(Left::nothing())
        },
        false,
    );
}

// ---- self-test ----

const FUNCTION_LEN: usize = 0x88;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x83, 0x3D, 0xEC, 0x54, 0x75, 0x00, 0x01];
const CALLS: [usize; 2] = [0x53, 0x6E];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Err(why) = check_move(
        &mut t,
        Direction::Right,
        rva(0x474766),
        FUNCTION_LEN,
        &ENTRY_BYTES,
        &CALLS,
    ) {
        return t.skip(why);
    }
    t.finish()
}
