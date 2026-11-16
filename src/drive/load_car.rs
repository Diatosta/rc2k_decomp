//! The current car's block into the work copy (0xFE0 bytes). See `drive/car.rs`. Refuses a null
//! block.

use crate::drive::car::*;
use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;

pub fn load_car(shared: &RaceShared) -> Result<(), Refused> {
    copy_car(current_block(shared)?, shared.car());

    Ok(())
}
