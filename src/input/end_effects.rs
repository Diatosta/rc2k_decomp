//! `input::end_effects` (0x40F2B8): the race's force feedback ended: both records'
//! effects asked to stop (`input::stop_effects`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. One caller, `input::device_command` (command 2, the race's
//! end). Its call through `part`. A call in its first five bytes: private copy (8 bytes,
//! the call relocated), manual patch. Shadow mode runs the original alone. Self-test: a
//! copy whose call goes to a noting stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::{Part, STOP, compare, dispatch_effects};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("input::end_effects", rva(0x40F2B8), thunk, &ORIGINAL)
    .private_copy(8)
    .relocate_calls(&[1])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// pusha; call input::stop_effects; popa; retn
const ENTRY: [u8; 8] = [0x60, 0xE8, 0xC7, 0xA3, 0xFF, 0xFF, 0x61, 0xC3];

/// The Rust version.
pub fn end_effects(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let mut g = *r;
    part(STOP, &mut g)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effects(&HOOK, r, call_original, end_effects);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_with(0x40F2B8, 8, &ENTRY, &[1], &[STOP], &[], &[], 0) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    compare(
        &mut t,
        "the call",
        &copy,
        Regs::SENTINEL,
        &|| {},
        &[],
        &|r, part| end_effects(r, part),
        false,
    );
    t.finish()
}
