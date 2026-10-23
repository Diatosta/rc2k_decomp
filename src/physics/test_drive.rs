//! `physics::test_drive` (0x4217E4): the car's figures from a test drive. Its specs
//! (`physics::load_car_specs`) and gear ratios (`physics::make_gear_ratios`); the drive
//! speeds, the car's speed, the gear, the engine and `unk_71A010` kept (pushed), the
//! drive's setting `dword_5CC190` into `unk_71A3CC`; `physics::test_drive_run`; the six
//! put back.
//!
//! In:  nothing
//! Out: as `test_drive_run` leaves the registers (ebx 5000); no flags read
//!
//! Caller: `scene::set_up_engine` (0x442AFB, the car's choice). Refuses what its parts refuse
//! (everything written put back). Hook removed 2026-10-22 (direct calls): Rust callers run its
//! `entry` (`calls::call`, `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{g, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The figures kept round the run: the drive speeds, the car's speed, the gear, the
/// engine, `unk_71A010`.
const KEPT: [usize; 6] = [0x71A190, 0x71A194, 0x71A184, 0x71A1AC, 0x71A1B0, 0x71A010];

/// The Rust version.
pub unsafe fn test_drive(entry: &Regs) -> Result<Left, Refused> {
    {
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
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { test_drive(r) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
