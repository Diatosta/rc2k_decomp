//! The work copy back into the current car's block (0xFE0 bytes). See `drive/car.rs`. Refuses a
//! null block.

use crate::drive::car::*;
use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;

pub fn store_car(shared: &RaceShared) -> Result<(), Refused> {
    copy_car(shared.car(), current_block(shared)?);

    Ok(())
}
