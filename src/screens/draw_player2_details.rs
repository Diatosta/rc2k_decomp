//! `screens::draw_player2_details` (0x4677B5): player 2's details page: the same code as player 1's.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `screens::draw_player_details`, whose code this is (`Layout::FULL`). In the draw list at 0x758CC2.
//! Compared in shadow mode. Self-test: the four players under four settings each.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::draw_player_details::{FULL, check_page, dispatch_page};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_player2_details",
    rva(0x4677B5),
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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_page(&HOOK, r, call_original, FULL);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    check_page(&mut t, call_original, FULL);
    t.finish()
}
