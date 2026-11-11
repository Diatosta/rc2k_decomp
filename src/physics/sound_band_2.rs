//! `physics::sound_band_2` (0x45D097): the car's `speed` x `contact_band_scale`, less 10, held to 0 ..= 0x40.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! Returns: the band
//!
//! Caller: `physics::place_sound_object`.

use crate::physics::sound_bands::Twin;

pub const TWIN: Twin = (
    0x45D097,
    |race| &race.shared.car.speed,
    |race| &race.frame.sound_spots.contact_band_scale,
    10,
);
