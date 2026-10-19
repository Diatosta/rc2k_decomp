//! `input::stop_effects` (0x409685): both device records' effects asked to stop
//! (`input::stop_device_effects` on `g_inputRecord1`, then `g_inputRecord2`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs` and `start_effects`. One caller, `input::end_effects`. Its calls
//! through `part`. Shadow mode runs the original alone. Self-test: a copy whose calls go
//! to the noting stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::input::effects::{Part, STOP_DEVICE, dispatch_effects};
use crate::input::start_effects::{both_records, test_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::stop_effects", rva(0x409685), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub fn stop_effects(r: &mut Regs, part: Part) -> Result<(), Refused> {
    both_records(r, STOP_DEVICE, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effects(&HOOK, r, call_original, stop_effects);
}

fn self_test() -> (usize, usize) {
    test_both(&HOOK, 0x409685, STOP_DEVICE, stop_effects)
}
