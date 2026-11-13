//! The current car set to 0 and `drive::store_car` (the work car back into the first block).
//! Refuses what `store_car` refuses.

use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;

/// Makes car 0 the current car.
pub fn store_first_car(shared: &RaceShared) -> Result<(), Refused> {
    shared.work.get().current_car.set(0);

    crate::drive::store_car::store_car(shared)?;
    Ok(())
}
