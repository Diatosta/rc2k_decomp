//! The current car's block into the work copy. See `drive/car.rs`. Refuses a null block.

use crate::drive::car::*;
use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;

pub fn load_car(shared: &RaceShared) -> Result<(), Refused> {
    let block = current_block(shared)?.clone();
    *shared.car_mut() = block;

    Ok(())
}
