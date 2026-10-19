//! `menu::music_volume_down` (0x467F48): the music's volume one less (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: as `option_steps` says (not `pusha`'d)
//! A left/right handler of the sound options page's items. Shadow mode runs the original
//! alone (it sends a sound command). Self-test: `option_steps::test_volume`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::option_steps::{MUSIC_LEVEL_RVA, Then, Volume, dispatch_volume, test_volume};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::music_volume_down", rva(0x467F48), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const STEP: Volume = Volume {
    address: 0x467F48,
    len: 0x29,
    entry: [0xFF, 0x0D, 0x88, 0x56, 0x75, 0x00],
    call: 0x23,
    value_rva: MUSIC_LEVEL_RVA,
    up: false,
    then: Then::Music,
};

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_volume(&HOOK, r, call_original, STEP);
}

fn self_test() -> (usize, usize) {
    test_volume(&HOOK, STEP)
}
