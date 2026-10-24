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
//! (everything written put back). Plain since 2026-10-23 (phase 3): its one caller calls it by
//! address (`routes.rs`), which sets the registers it leaves (`LEFT`).

use crate::errors::Refused;
use crate::physics::flow::{g, p};
use crate::physics::test_drive_run;

/// The figures kept round the run: the drive speeds, the car's speed, the gear, the
/// engine, `unk_71A010`.
const KEPT: [usize; 6] = [0x71A190, 0x71A194, 0x71A184, 0x71A1AC, 0x71A1B0, 0x71A010];

/// What the original leaves in eax, ebx, ecx and edx (`test_drive_run`'s last step:
/// 4999 / 20, the step count, 20, 4999 % 20). Its caller by address (`scene::set_up_engine`)
/// leaves ebx to `menu::main_menu_loop`, which reads it (0x47493E).
pub const LEFT: [u32; 4] = [249, test_drive_run::STEPS, 20, 19];

/// The Rust version.
pub unsafe fn test_drive() -> Result<(), Refused> {
    unsafe {
        crate::physics::load_car_specs::load_car_specs()?;
        crate::physics::make_gear_ratios::make_gear_ratios()?;
    }
    let kept = KEPT.map(g);
    p(0x71A3CC, g(0x5CC190));
    test_drive_run::test_drive_run()?;
    for (x, v) in KEPT.iter().zip(kept) {
        p(*x, v);
    }
    Ok(())
}
