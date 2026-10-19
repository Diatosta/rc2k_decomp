//! `menu::menu_sounds_a_volume_down` (0x468080): the first menu sounds' volume one less, heard (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: as `option_steps` says (not `pusha`'d)
//! A left/right handler of the sound options page's items. Shadow mode runs the original
//! alone (it sends a sound command). Self-test: `option_steps::test_volume`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::option_steps::{SOUNDS_A_RVA, Then, Volume, dispatch_volume, test_volume};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::menu_sounds_a_volume_down",
    rva(0x468080),
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

const STEP: Volume = Volume {
    address: 0x468080,
    len: 0x2E,
    entry: [0xFF, 0x0D, 0x9C, 0x56, 0x75, 0x00],
    call: 0x28,
    value_rva: SOUNDS_A_RVA,
    up: false,
    then: Then::Sound(0x10),
};

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_volume(&HOOK, r, call_original, STEP);
}

fn self_test() -> (usize, usize) {
    test_volume(&HOOK, STEP)
}
