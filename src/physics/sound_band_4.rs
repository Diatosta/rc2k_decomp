//! `physics::sound_band_4` (0x45D121): `g_car +0x1C4` x `flt_6D1AE4`, less 1, held to 0 ..= 0x40, in ebx.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! In:  nothing
//! Out: ebx the band; the rest kept
//!
//! Caller: `sub_45CF95`.
//! Self-test: `sound_bands::test_twin`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::sound_bands::Twin;

pub const TWIN: Twin = (0x45D121, 0x71A19C, 0x6D1AE4, 1);
