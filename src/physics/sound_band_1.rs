//! `physics::sound_band_1` (0x45D067): `contact_speed_length` x `contact_band_scale`, less 1,
//! held to 0 ..= 0x40. See `physics/sound_bands.rs` (the core and the test).
//!
//! Returns: the band
//!
//! Caller: `physics::place_sound_e1`.

use crate::physics::sound_bands::Twin;

pub const TWIN: Twin = (
    0x45D067,
    |c| c.contact.contact_speed_length.bits(),
    |c| c.spots.contact_band_scale.bits(),
    1,
);
