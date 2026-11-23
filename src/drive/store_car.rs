//! The work copy back into the current car's block. See `drive/car.rs`. Refuses a null block.

use crate::drive::car::*;
use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;

pub fn store_car(shared: &RaceShared) -> Result<(), Refused> {
    let car = shared.car().clone();
    *current_block_mut(shared)? = car;

    Ok(())
}
