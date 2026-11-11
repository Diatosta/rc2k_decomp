//! `physics::sound_band_4` (0x45D121): the car's `speed` x `crash_band_scale`, less 1, held to 0 ..= 0x40.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! Returns: the band
//!
//! Caller: `physics::place_sound_e5`.

use crate::physics::sound_bands::Twin;

pub const TWIN: Twin = (
    0x45D121,
    |race| &race.shared.car.speed,
    |race| &race.frame.sound_spots.crash_band_scale,
    1,
);
