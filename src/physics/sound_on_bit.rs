//! When the given bit of `parts_present` is set, it is cleared and `physics::place_car_sound` runs.
//!
//! Refuses what `place_car_sound` refuses (before anything: the bit is cleared after the check).

use crate::errors::Refused;
use crate::physics::place_car_sound::place_car_sound;
use crate::race::matrices_store::ViewKept;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::parts::take_bit;

// Core: `rc2k_core::physics::parts::take_bit`.
pub(super) fn sound_on_bit(
    numbers: &Numbers,
    matrices: &ViewKept,
    shared: &RaceShared,
    cl: u32,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let (_, cleared) = take_bit(shared.car().parts_present.get(), cl);
    let Some(cleared) = cleared else {
        return Ok(());
    };
    shared.car().parts_present.set(cleared);
    { place_car_sound(numbers, matrices, shared, sound) }?;
    Ok(())
}
