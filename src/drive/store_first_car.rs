//! `drive::store_first_car` (0x460CAA): `g_currentCar` 0 and `drive::store_car` (the work
//! car back into the first block).
//!
//! In:  nothing
//! Out: as `store_car` leaves the registers; no flags read
//!
//! Callers: `drive::finish_run`, `sub_4293CB` (twice). Refuses what `store_car` refuses.

use crate::errors::Refused;

/// The Rust version.
/// The Rust version. The registers and ZF it leaves are read by no caller (liveness: ZF
/// after `sub_4293CB`'s call has no read point).
pub fn store_first_car() -> Result<(), Refused> {
    crate::race::owner::race().shared.work.current_car.set(0);
    crate::drive::store_car::store_car()?;
    Ok(())
}
