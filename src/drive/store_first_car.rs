//! `drive::store_first_car` (0x460CAA): `g_currentCar` 0 and `drive::store_car` (the work car back
//! into the first block).
//!
//! Callers: `drive::finish_run`, 0x4293CB (twice). Refuses what `store_car` refuses.

use crate::errors::Refused;
use crate::race::owner_shared::Shared;

/// The Rust version.
pub fn store_first_car(shared: &Shared) -> Result<(), Refused> {
    shared.work.current_car.set(0);

    crate::drive::store_car::store_car(shared)?;
    Ok(())
}
