//! `physics::reset_when_stuck` (0x43EFBA): with the grades' mode (`dword_71A234`, `g_car`
//! +0x25C) 0 or 2 the stuck count (+0x1E8, `dword_71A1C0`) is set to 0; else it goes up,
//! and at 700 or more (unsigned) `physics::reset_car` puts the car back on the line.
//!
//! In:  nothing
//! Out: when `reset_car` ran, the registers as it leaves them; else all kept; no flags
//!      read (the caller's next is a call)
//!
//! Caller: `sub_43B635` (0x43B900), every physics step. Refuses what `reset_car` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use rc2k_core::physics::start::stuck_tick;

/// The Rust version. The registers it leaves are read by none of its callers.
// Converted: `rc2k_core::physics::start::stuck_tick`.
pub unsafe fn reset_when_stuck() -> Result<(), Refused> {
    unsafe {
        let mode: u32 = CAR.driver_kind.bits();
        let mut stuck = if mode == 2 || mode == 0 {
            0
        } else {
            CAR.stuck_count.get()
        };
        let reset = stuck_tick(mode, &mut stuck);
        CAR.stuck_count.set(stuck);
        if reset {
            crate::physics::reset_car::reset_car()?;
        }
    }
    Ok(())
}
