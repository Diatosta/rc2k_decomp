//! `marks::emit_particle_22` (0x4CE482): `marks::emit_particle_19`'s code on its own
//! variables: kind 0x16, the level the car's +0x4C4, the flags 0x33, `dword_93A4B2`,
//! `flt_93A4B6` .. `flt_93A4D2`, `dword_93A4D6` (an instruction diff: nothing else
//! differs).
//!
//! In:  esi the car object
//! Out: every register kept
//!
//! Refuses as `emit_particle_19`. Self-test: as `emit_particle_19`'s.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::emit_particle_19::{TwinSet, emit_twin, regions, test_with};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::emit_particle_22", rva(0x4CE482), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `emit_particle_22`'s variables.
pub const TWENTY_SECOND: TwinSet = TwinSet {
    kind: 0x16,
    level_at: 0x4C4,
    flags: 0x33,
    chance_random: rva(0x93A4B2),
    lift: rva(0x93A4B6),
    row_k: [rva(0x93A4BA), rva(0x93A4BE), rva(0x93A4C2)],
    spread: rva(0x93A4C6),
    motion_k: [rva(0x93A4CA), rva(0x93A4CE), rva(0x93A4D2)],
    spin_random: rva(0x93A4D6),
};

/// The Rust version (see the header).
pub unsafe fn emit_particle_22(object: u32) -> Result<(), Refused> {
    unsafe { emit_twin(&TWENTY_SECOND, object) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_particle_22(r.esi) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(&TWENTY_SECOND), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_with(&mut t, &TWENTY_SECOND, call_original, &rust);
    t.finish()
}
