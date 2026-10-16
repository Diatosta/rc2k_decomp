//! `input::update` (0x420115): the frame's input, the mouse and the keys pressed.
//!
//! In:  nothing
//! Out: eax the right click, ebx 0x80, ecx 0, edx the last key dword, esi `g_keysNow`,
//!      edi `g_keysBefore`; ebp preserved
//!
//! See `input/keys.rs`. Seven callers (`sub_424001` and the race's and its menus' frame
//! loops). Self-test: the original and the Rust version on the game's input state, set
//! and put back (never with the screenshot key down).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::keys::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::update", rva(0x420115), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        update(r, &mut game_screenshot)
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
        (3u32, 0x0020_0503u32),
        (0x99, 0xFF00_0001),
        (0x4242, 0x7FF7_FF02),
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
