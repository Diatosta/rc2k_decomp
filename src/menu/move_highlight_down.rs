//! `menu::move_highlight_down` (0x474715): moves the highlight along the items' link at +0x28, on
//! key 0x6B.
//!
//! In:  nothing (edi is the menu at its one call, and is not used)
//! Out: every register preserved
//! It goes on along the link until it reaches an item that can be chosen. "Down" is a reading.
//! The rules, the click and the item's callback (+0x38) are in `menu/navigation.rs`.
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

pub static HOOK: RawHook =
    RawHook::new("menu::move_highlight_down", rva(0x474715), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version: `navigation::move_highlight` with `Direction::Down` ----

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_outside(
        &HOOK,
        r,
        call_original,
        &move_regions(),
        &|r: &Regs, out: &mut dyn Outside| {
            unsafe { move_highlight(Direction::Down, r, out) }?;
            Ok(Left::nothing())
        },
        false,
    );
}

// ---- self-test ----

const FUNCTION_LEN: usize = 0x51;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8B, 0x1D, 0x25, 0x55, 0x75, 0x00, 0xE8];
const CALLS: [usize; 4] = [0x07, 0x19, 0x31, 0x3F];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Err(why) = check_move(
        &mut t,
        Direction::Down,
        rva(0x474715),
        FUNCTION_LEN,
        &ENTRY_BYTES,
        &CALLS,
    ) {
        return t.skip(why);
    }
    t.finish()
}
