//! `physics::sound_band_1` (0x45D067): `flt_6D3108` x `flt_6D1AE0`, less 1, held to 0 ..= 0x40, in ebx.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! In:  nothing
//! Out: ebx the band; the rest kept
//!
//! Caller: `sub_45CEF9`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::sound_bands::Twin;
use crate::race::sound_spots_store::RACE_SOUND_SPOTS;

pub const TWIN: Twin = (
    0x45D067,
    || &CONTACT_WORK.contact_speed_length,
    || &RACE_SOUND_SPOTS.band_scale_1,
    1,
);
