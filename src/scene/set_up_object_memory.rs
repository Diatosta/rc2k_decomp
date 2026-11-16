//! `scene::set_up_object_memory`: the objects' memory and the kinds' records.
//!
//! `scene::allocate_object_memory`, then `scene::read_kind_records`.

use crate::errors::Refused;
use crate::scene::allocate_object_memory::allocate_object_memory_in_game;
use crate::scene::read_kind_records::{read_kind_records, store_kind_records};

/// The objects' memory allocated, then the kinds' records read and stored.
pub fn set_up_object_memory(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    allocate_object_memory_in_game(&race.course)?;
    let records = read_kind_records(race, scene)?;
    store_kind_records(race, &records)?;
    Ok(())
}
