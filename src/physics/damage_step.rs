//! `physics::damage_step` (0x4AB5D6): the car's damage and wear model for this physics
//! step: `physics::damage_factors`, `physics::damage_wear`, `physics::damage_gauges`
//! (its only code: three calls and `retn`).
//!
//! In:  nothing
//! Out: the registers as the three leave them; no flags read
//!
//! Caller: `sub_43B635` (0x43B952), every physics step. It begins with a call: hooked by
//! a private copy of its 0x10 bytes (the three calls relocated) and a hand-made patch.
//! Self-test: `physics/damage_model.rs` (`damage_test`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::physics::damage_model::{damage_test, regions};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("physics::damage_step", rva(0x4AB5D6), thunk, &ORIGINAL)
    .private_copy(0x10)
    .relocate_calls(&[0x0, 0x5, 0xA])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn damage_step(entry: &Regs) -> Left {
    let mut r = *entry;
    crate::physics::damage_factors::steps(&mut r);
    crate::physics::damage_wear::steps(&mut r);
    crate::physics::damage_gauges::steps(&mut r);
    left_of(&r)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { damage_step(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    damage_test(&HOOK, call_original, &rust)
}
