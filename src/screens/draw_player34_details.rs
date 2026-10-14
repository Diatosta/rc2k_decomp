//! `screens::draw_player34_details` (0x467975): players 3 and 4's details page: player 1's without the second view (the
//! controller's item 8, the lines below the view 0x1B higher).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `screens::draw_player_details`, whose code this is (`Layout::SHORT`). In the draw lists at 0x758CCA and 0x758CD2.
//! Compared in shadow mode. Self-test: the four players under four settings each.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::draw_player_details::{SHORT, check_page, dispatch_page};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_player34_details",
    rva(0x467975),
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
    dispatch_page(&HOOK, r, call_original, SHORT);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    check_page(&mut t, call_original, SHORT);
    t.finish()
}
