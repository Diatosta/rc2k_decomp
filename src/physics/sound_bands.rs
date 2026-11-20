//! The four bands 0 ..= 0x40 the car's sounds are placed with: a value times a factor (rounded
//! through `band_scratch`), less a step, held to 0 ..= 0x40. A reading: volume or pitch steps of
//! the contact, object, bump and crash sounds.

use crate::math::x87;
use crate::memory::session::PlainDword;
use crate::physics::step_families::SoundInputs;
use rc2k_core::physics::sound_bands::sound_band;

/// What a band is made of.
pub struct Band {
    pub value: fn(&SoundInputs<'_>) -> f32,
    pub factor: fn(&SoundInputs<'_>) -> f32,
    pub step: u32,
}

/// The wall and ground contacts' band: the contact's speed.
pub const CONTACT_BAND: Band = Band {
    value: |c| c.contact.contact_speed_length.get(),
    factor: |c| c.spots.contact_band_scale.get(),
    step: 1,
};

/// A hit object's band: the car's speed.
pub const OBJECT_BAND: Band = Band {
    value: |c| c.car.speed.get(),
    factor: |c| c.spots.contact_band_scale.get(),
    step: 10,
};

/// A bump between cars: the bump's speed.
pub const BUMP_BAND: Band = Band {
    value: |c| c.contact.bump_speed_length.get(),
    factor: |c| c.spots.bump_band_scale.get(),
    step: 10,
};

/// A crash: the car's speed.
pub const CRASH_BAND: Band = Band {
    value: |c| c.car.speed.get(),
    factor: |c| c.spots.crash_band_scale.get(),
    step: 1,
};

/// The exact x87 version on the value and the factor; the scratch keeps the rounded number.
fn band_at(value: f32, factor: f32, scratch: &PlainDword, step: u32) -> u32 {
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
pub(super) fn band_value(c: &SoundInputs<'_>, band: &Band) -> u32 {
    band_at(
        (band.value)(c),
        (band.factor)(c),
        &c.spots.band_scratch,
        band.step,
    )
}

/// The converted version (`rc2k_core::physics::sound_bands`, plain floats).
// Core: `rc2k_core::physics::sound_bands::sound_band`.
fn band_at_plain(value: f32, factor: f32, scratch: &PlainDword, step: u32) -> u32 {
    let (n, b) = sound_band(value, factor, step);
    scratch.set(n as u32);
    b
}

/// The converted band on the game's variables.
pub(super) fn band_value_plain(c: &SoundInputs<'_>, band: &Band) -> u32 {
    band_at_plain(
        (band.value)(c),
        (band.factor)(c),
        &c.spots.band_scratch,
        band.step,
    )
}
