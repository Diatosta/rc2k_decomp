//! Four twins that make a band 0 ..= 0x40 out of a float: value x factor (rounded through
//! `band_scratch`), less a step, held to 0 ..= 0x40 (by the sign):
//! - `physics::sound_band_1` (0x45D067): `contact_speed_length` x `contact_band_scale`, less 1;
//! - `physics::sound_band_2` (0x45D097): the car's `speed` x `contact_band_scale`, less 10;
//! - `physics::sound_band_3` (0x45D0F1): `bump_speed_length` x `bump_band_scale`, less 10;
//! - `physics::sound_band_4` (0x45D121): the car's `speed` x `crash_band_scale`, less 1.
//!
//! A reading: volume or pitch steps for the car's sounds (their callers
//! `physics::place_sound_object` ..
//! `physics::place_sound_e5` drive the sound objects).
//!
//! Returns: the band

use crate::math::x87;
use crate::physics::step_families::SoundInputs;
use crate::session::{Dword, Float};
use rc2k_core::physics::sound_bands::sound_band;

/// One twin: (address, value, factor, step).
pub type Twin = (
    usize,
    for<'a> fn(&'a SoundInputs<'a>) -> &'a Float,
    for<'a> fn(&'a SoundInputs<'a>) -> &'a Float,
    u32,
);

/// The exact x87 version on the value and the factor, the scratch keeps the number of `fistp`.
pub fn band_at(value: &Float, factor: &Float, scratch: &Dword, step: u32) -> u32 {
    let n = x87::mul_to_int(value, factor) as u32;
    scratch.set(n);
    let mut b = n.wrapping_sub(step);
    if (b as i32) < 0 {
        b = 0;
    }
    if (b.wrapping_sub(0x40) as i32) >= 0 {
        b = 0x40;
    }
    b
}

/// The band on the game's variables (the exact x87 version).
pub fn band_value(c: &SoundInputs<'_>, twin: Twin) -> u32 {
    band_at(twin.1(c), twin.2(c), &c.spots.band_scratch, twin.3)
}

/// The converted version (`rc2k_core::physics::sound_bands`, plain floats).
// Converted: `rc2k_core::physics::sound_bands::sound_band`.
pub fn band_at_plain(value: &Float, factor: &Float, scratch: &Dword, step: u32) -> u32 {
    let (n, b) = sound_band(value.get(), factor.get(), step);
    scratch.set(n as u32);
    b
}

/// The converted band on the game's variables.
pub fn band_value_plain(c: &SoundInputs<'_>, twin: Twin) -> u32 {
    band_at_plain(twin.1(c), twin.2(c), &c.spots.band_scratch, twin.3)
}
