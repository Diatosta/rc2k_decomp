//! `drive::store_first_car` (0x460CAA): `g_currentCar` 0 and `drive::store_car` (the work
//! car back into the first block).
//!
//! In:  nothing
//! Out: as `store_car` leaves the registers; no flags read
//!
//! Callers: `drive::finish_run`, `sub_4293CB` (twice).
//! Refuses what `store_car` refuses.
//! Self-test: the original and the Rust version on the work car and the first block
//! (saved and put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::physics::flow::p;

/// The Rust version.
/// The Rust version. The registers and ZF it leaves are read by no caller (liveness: ZF
/// after `sub_4293CB`'s call has no read point).
pub unsafe fn store_first_car() -> Result<(), Refused> {
    p(0x71B3F8, 0);
    unsafe { crate::drive::store_car::store_car() }?;
    Ok(())
}
