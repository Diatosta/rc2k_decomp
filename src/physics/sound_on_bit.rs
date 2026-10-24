//! `physics::sound_on_bit` (0x441324): when bit cl of `g_car` +0x298 (`unk_71A270`) is
//! set, it is cleared and `physics::place_car_sound` runs.
//!
//! In:  cl the bit
//! Out: eax 1 << cl; the rest kept; no flags read (the callers load floats next)
//!
//! Callers: `sub_4411EC` (four). Refuses what `place_car_sound` refuses (before anything:
//! the bit is cleared after the check).

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::physics::car_sounds::object;
use crate::physics::place_car_sound::place_car_sound;
use rc2k_core::physics::parts::take_bit;

const BITS_RVA: usize = rva(0x71A270); // g_car + 0x298

/// The Rust version.
// Converted: `rc2k_core::physics::parts::take_bit`.
pub unsafe fn sound_on_bit(cl: u32) -> Result<(), Refused> {
    let (_, cleared) = take_bit(unsafe { game_read(BITS_RVA) }, cl);
    let Some(cleared) = cleared else {
        return Ok(());
    };
    object()?;
    unsafe { game_write(BITS_RVA, cleared) };
    unsafe { place_car_sound() }?;
    Ok(())
}
