//! `physics::sound_band_1` (0x45D067): `flt_6D3108` x `flt_6D1AE0`, less 1, held to 0 ..= 0x40, in ebx.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! Returns: the band
//!
//! Caller: `physics::place_sound_e1`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::sound_bands::Twin;

pub const TWIN: Twin = (
    0x45D067,
    |race| &race.physics.contact.contact_speed_length,
    |race| &race.frame.sound_spots.band_scale_1,
    1,
);
