//! `physics::test_drive` (0x4217E4): the car's figures from a test drive. Its specs
//! (`physics::load_car_specs`) and gear ratios (`physics::make_gear_ratios`); the drive
//! speeds, the car's speed, the gear, the engine and `unk_71A010` kept (pushed), the
//! drive's setting `dword_5CC190` into `unk_71A3CC`; `physics::test_drive_run`; the six
//! put back.
//!
//! In:  nothing
//! Out: as `test_drive_run` leaves the registers (ebx 5000); no flags read
//!
//! Caller: `scene::set_up_engine` (0x442AFB, the car's choice). It begins with a call:
//! hooked by a private copy of its 0x64 bytes with the three calls relocated, and a
//! hand-made patch.
//! Refuses what its parts refuse (everything written put back).
//! Self-test: `physics/test_drive_run.rs` (`drive_test`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::physics::flow::{g, guard, p};
use crate::physics::test_drive_run::{drive_test, regions};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("physics::test_drive", rva(0x4217E4), thunk, &ORIGINAL)
    .private_copy(0x64)
    .relocate_calls(&[0x0, 0x5, 0x3A])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The figures kept round the run: the drive speeds, the car's speed, the gear, the
/// engine, `unk_71A010`.
const KEPT: [usize; 6] = [0x71A190, 0x71A194, 0x71A184, 0x71A1AC, 0x71A1B0, 0x71A010];

/// The Rust version.
pub unsafe fn test_drive(entry: &Regs) -> Result<Left, Refused> {
    guard(&regions(), || {
        let mut r = *entry;
        unsafe {
            crate::physics::load_car_specs::load_car_specs()?.apply(&mut r);
            r.ebx = crate::physics::make_gear_ratios::make_gear_ratios()?;
        }
        let kept = KEPT.map(g);
        p(0x71A3CC, g(0x5CC190));
        let run = unsafe { crate::physics::test_drive_run::test_drive_run(&r) }?;
        run.apply(&mut r);
        for (x, v) in KEPT.iter().zip(kept) {
            p(*x, v);
        }
        Ok(crate::race::stage_prefs::left_of(&r))
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { test_drive(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    drive_test(&HOOK, call_original, &rust)
}
