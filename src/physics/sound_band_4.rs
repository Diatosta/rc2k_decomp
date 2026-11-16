//! The car's `speed` x `crash_band_scale`, less 1, held to 0 ..= 0x40. See `physics/sound_bands.rs`
//! (the core and the test).
//!
//! Caller: `physics::place_sound_e5`.

use crate::physics::sound_bands::Twin;

pub const TWIN: Twin = (
    0x45D121,
    |c| c.car.speed.get(),
    |c| c.spots.crash_band_scale.get(),
    1,
);
