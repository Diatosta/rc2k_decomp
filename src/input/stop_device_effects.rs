//! `input::stop_device_effects` (0x40969E): the six effects of the record in esi asked to
//! stop (`input::set_effect` with ebx 4, eax 0 .. 5).
//!
//! In:  esi = the device's record
//! Out: eax 6, ebx 4, the rest as `set_effect` keeps them (all)
//!
//! See `input/effects.rs` and `start_device_effects` (the same loop with 2). Two callers,
//! both in `input::stop_effects`. Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to the noting stand-ins, against the Rust
//! version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::input::effects::{Part, dispatch_effects};
use crate::input::start_device_effects::{all_six, test_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "input::stop_device_effects",
    rva(0x40969E),
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

/// The Rust version.
pub fn stop_device_effects(r: &mut Regs, part: Part) -> Result<(), Refused> {
    all_six(r, 4, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effects(&HOOK, r, call_original, stop_device_effects);
}

fn self_test() -> (usize, usize) {
    // xor eax, eax; mov ebx, 4
    test_both(
        &HOOK,
        0x40969E,
        &[0x33, 0xC0, 0xBB, 0x04, 0x00, 0x00, 0x00],
        stop_device_effects,
    )
}
