//! The four bands 0 ..= 0x40 the car's sounds are placed with: a value times a factor (rounded), less a step, held to 0 ..= 0x40. A reading: volume or pitch steps of
//! the contact, object, bump and crash sounds.

use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::step_families::SoundInputs;
use rc2k_core::physics::sound_bands::sound_band;

/// The factor of the wall / ground contact sound and of the object band.
const CONTACT_BAND_SCALE: f32 = 50.0;
/// The factor of the car-bump sound.
const BUMP_BAND_SCALE: f32 = 200.0;
/// The factor of the crash sound.
const CRASH_BAND_SCALE: f32 = 50.0;

/// What a band is made of.
pub struct Band {
    pub value: fn(&SoundInputs<'_>, &Car) -> f32,
    pub factor: fn(&SoundInputs<'_>) -> f32,
    pub step: u32,
}

/// The wall and ground contacts' band: the contact's speed.
pub const CONTACT_BAND: Band = Band {
    value: |c, _| f32::from_bits(c.contact.borrow().contact_speed_length),
    factor: |_| CONTACT_BAND_SCALE,
    step: 1,
};

/// A hit object's band: the car's speed.
pub const OBJECT_BAND: Band = Band {
    value: |_, car| car.speed,
    factor: |_| CONTACT_BAND_SCALE,
    step: 10,
};

/// The step of a car-bump band.
const BUMP_BAND_STEP: u32 = 10;

/// A crash: the car's speed.
pub const CRASH_BAND: Band = Band {
    value: |_, car| car.speed,
    factor: |_| CRASH_BAND_SCALE,
    step: 1,
};

/// The exact x87 version on the value and the factor.
fn band_at(value: f32, factor: f32, step: u32) -> u32 {
    let n = x87::mul_to_int(value, factor) as u32;
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
pub(super) fn band_value(c: &SoundInputs<'_>, band: &Band, car: &Car) -> u32 {
    band_at((band.value)(c, car), (band.factor)(c), band.step)
}

/// The converted version (`rc2k_core::physics::sound_bands`, plain floats).
// Core: `rc2k_core::physics::sound_bands::sound_band`.
fn band_at_plain(value: f32, factor: f32, step: u32) -> u32 {
    sound_band(value, factor, step).1
}

/// The converted band on the game's variables.
pub(super) fn band_value_plain(c: &SoundInputs<'_>, band: &Band, car: &Car) -> u32 {
    band_at_plain((band.value)(c, car), (band.factor)(c), band.step)
}

/// The converted band of a bump between cars: the bump's speed.
pub(super) fn bump_band_plain(speed: f32) -> u32 {
    band_at_plain(speed, BUMP_BAND_SCALE, BUMP_BAND_STEP)
}
