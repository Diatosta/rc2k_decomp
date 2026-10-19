//! `input::effect_5_change` (0x4098BF): the function of a device's effect 5 (see
//! `input/effect_changes.rs`).
//!
//! In:  eax = the change, ecx and edx its values, esi the device's record, edi the effect
//! Out: every register kept
//! Called through the records' effects by `input::device_effects`. Shadow mode runs the
//! original alone. Manual patch with a private copy (a `jnz` in the first bytes, its calls
//! relocated). Self-test: `effect_changes::test_effect`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::input::effect_changes::{Effect, dispatch_effect, test_effect};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("input::effect_5_change", rva(0x4098BF), thunk, &ORIGINAL)
    .private_copy(0x70)
    .relocate_calls(&[0x5D, 0x69])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EFFECT: Effect = Effect {
    number: 5,
    address: 0x4098BF,
    len: 0x70,
    entry: [0x60, 0x83, 0xF8, 0x01, 0x75, 0x3A],
    calls: &[0x5D, 0x69],
};

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effect(&HOOK, r, call_original, 5);
}

fn self_test() -> (usize, usize) {
    test_effect(&HOOK, EFFECT)
}
