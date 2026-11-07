//! `physics::sound_on_bit` (0x441324): when bit cl of `g_car` +0x298 (`unk_71A270`) is
//! set, it is cleared and `physics::place_car_sound` runs.
//!
//! Takes: the bit
//!
//! Callers: `physics::sound_hit_parts` (four). Refuses what `place_car_sound` refuses (before
//! anything: the bit is cleared after the check).

use crate::errors::Refused;
use crate::physics::place_car_sound::place_car_sound;
use crate::race::owner::Race;
use rc2k_core::physics::parts::take_bit;

/// The Rust version.
// Converted: `rc2k_core::physics::parts::take_bit`.
pub fn sound_on_bit(race: &Race, cl: u32) -> Result<(), Refused> {
    let (_, cleared) = take_bit(race.shared.car.parts_present.bits(), cl);
    let Some(cleared) = cleared else {
        return Ok(());
    };
    race.shared.car.parts_present.set(cleared);
    { place_car_sound(race) }?;
    Ok(())
}
