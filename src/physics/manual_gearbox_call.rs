//! `physics::manual_gearbox_call` (0x43C2A8): `call physics::manual_gearbox; retn`.
//!
//! In:  nothing
//! Out: as `manual_gearbox` leaves them
//!
//! Caller: `physics::gearbox_step` (0x43C1AD).
//! Refuses what `manual_gearbox` refuses.
//! Self-test: `physics/gearbox.rs` (`test_gearbox`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::gearbox::{guarded, regions, test_gearbox};
use crate::physics::manual_gearbox::manual_gearbox_in;
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::manual_gearbox_call",
    rva(0x43C2A8),
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
pub unsafe fn manual_gearbox_call(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &manual_gearbox_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { manual_gearbox_call(r) }
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
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_gearbox(&mut t, call_original, &rust, &|_| {});
    t.finish()
}
