//! `physics::sound_band_3` (0x45D0F1): `bump_speed_length` x `bump_band_scale`, less 10, held to 0 ..= 0x40.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! Returns: the band
//!
//! Caller: `physics::place_sound_e1_other`.

use crate::physics::sound_bands::Twin;

pub const TWIN: Twin = (
    0x45D0F1,
    |race| &race.physics.contact.bump_speed_length,
    |race| &race.frame.sound_spots.bump_band_scale,
    10,
);
