//! `marks::emit_bursts_2` (0x4CD747): `marks::emit_bursts` on its own factors and randoms
//! (`flt_93A3F6`, `flt_93A406`, `flt_93A40A`, `dword_93A3FA` .. `dword_93A416`); the code
//! is the same instruction for instruction, the ring-full slip included (0x4CDC6E's
//! exit; section 16).
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! See `marks/emit_bursts.rs`; no variable of its own shared but the entries' and the
//! scratch ones. Caller: `marks::emit_wheel_particles`' kind 3. Refuses as
//! `emit_bursts`. Self-test: as `emit_bursts`'.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::emit_bursts::{BurstSet, FIRST, Half, emit_bursts_with, regions, test_with};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::emit_bursts_2", rva(0x4CD747), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `emit_bursts_2`'s: `emit_bursts`' with its own factors and randoms.
pub const SECOND: BurstSet = BurstSet {
    halves: [
        Half {
            rate_random: rva(0x93A3FA),
            count_random: rva(0x93A3FE),
            entry_random: rva(0x93A402),
            ..FIRST.halves[0]
        },
        Half {
            rate_random: rva(0x93A40E),
            count_random: rva(0x93A412),
            entry_random: rva(0x93A416),
            ..FIRST.halves[1]
        },
    ],
    strength_k: rva(0x93A3F6),
    blend_k: [rva(0x93A406), rva(0x93A40A)],
};

/// The Rust version (see the header).
pub unsafe fn emit_bursts_2(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe { emit_bursts_with(&SECOND, wheel, object) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_bursts_2(r.edi, r.esi) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(&SECOND, r.edi), &rust);
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
    test_with(&mut t, &SECOND, call_original, &rust);
    t.finish()
}
