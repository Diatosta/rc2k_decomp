//! `physics::sound_on_bit` (0x441324): when bit cl of `g_car` +0x298 (`unk_71A270`) is
//! set, it is cleared and `physics::place_car_sound` runs.
//!
//! In:  cl the bit
//! Out: eax 1 << cl; the rest kept; no flags read (the callers load floats next)
//!
//! Callers: `sub_4411EC` (four). Refuses what `place_car_sound` refuses (before anything:
//! the bit is cleared after the check).

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::place_car_sound::place_car_sound;
use rc2k_core::physics::parts::take_bit;

/// The Rust version.
// Converted: `rc2k_core::physics::parts::take_bit`.
pub fn sound_on_bit(cl: u32) -> Result<(), Refused> {
    let (_, cleared) = take_bit(CAR.parts_present.bits(), cl);
    let Some(cleared) = cleared else {
        return Ok(());
    };
    CAR.parts_present.set(cleared);
    { place_car_sound() }?;
    Ok(())
}
