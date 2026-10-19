//! `input::effect_3_change` (0x4097B4): the function of a device's effect 3 (see
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

pub static HOOK: RawHook = RawHook::new("input::effect_3_change", rva(0x4097B4), thunk, &ORIGINAL)
    .private_copy(0x46)
    .relocate_calls(&[0x33, 0x3F])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EFFECT: Effect = Effect {
    number: 3,
    address: 0x4097B4,
    len: 0x46,
    entry: [0x60, 0x83, 0xF8, 0x01, 0x75, 0x10],
    calls: &[0x33, 0x3F],
};

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effect(&HOOK, r, call_original, 3);
}

fn self_test() -> (usize, usize) {
    test_effect(&HOOK, EFFECT)
}
