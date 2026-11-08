//! `physics::sound_band_2` (0x45D097): `g_car +0x1C4` x `flt_6D1AE0`, less 10, held to 0 ..= 0x40, in ebx.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! Returns: the band
//!
//! Caller: `physics::place_sound_object`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::sound_bands::Twin;

pub const TWIN: Twin = (
    0x45D097,
    |race| &race.shared.car.speed,
    |race| &race.frame.sound_spots.band_scale_1,
    10,
);
