//! `input::take_mouse_clicks` (0x42028C): the mouse's clicks from the keys pressed.
//!
//! In:  nothing
//! Out: eax = the right click; the rest preserved
//!
//! See `input/keys.rs`. One caller, `input::update`. Self-test: the original and the Rust
//! version on the game's input state, set and put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::keys::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::take_mouse_clicks", rva(0x42028C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        take_mouse_clicks(r)
    }))
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
    for (seed, mouse) in [
        (1u32, 0x1234_5603u32),
        (7, 0xFFF8_0102),
        (0x55, 0x8000_0300),
        (0x1000, 0),
    ] {
        check_state(
            &mut t,
            &format!("seed {seed:#x}, mouse {mouse:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| set_state(seed, mouse),
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
