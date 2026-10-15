//! `menu::hide_in_game_options` (0x46FB63): the in-game options' last items hidden.
//!
//! In:  eax = 1, 2 or 3 (anything else: nothing)
//! Out: every register preserved
//!
//! See `menu/in_game_options_items.rs`: items 3 + eax .. 6 get no string, are passed over
//! (+0x48 0xFFFFFC1C) and have nothing behind (+0x1C 0xFFFFFFA9). Callers: the event loops
//! (`sub_482382` with 1) and seven more. Self-test: the game's menu, values 0 .. 4, put
//! back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::in_game_options_items::{hide_in_game_options, items_region};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

// A `jz` at +4 is inside the first 5 bytes: the jump is written by hand and the original
// runs from a private copy of the whole function (no calls, every jump inside it).
pub static HOOK: RawHook = RawHook::new(
    "menu::hide_in_game_options",
    rva(0x46FB63),
    thunk,
    &ORIGINAL,
)
.private_copy(0x80)
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    hide_in_game_options(r.eax);
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[items_region()], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [items_region()];
    let saved = capture(&regions);
    for from in 0..=4u32 {
        check_state(
            &mut t,
            &format!("eax {from}"),
            call_original,
            Regs {
                eax: from,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::write_bytes(regions[0].0 as *mut u8, 0xAA, regions[0].1) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
