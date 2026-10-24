//! `physics::sound_band_3` (0x45D0F1): `flt_6D3CD0` x `flt_6D1AE8`, less 10, held to 0 ..= 0x40, in ebx.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! In:  nothing
//! Out: ebx the band; the rest kept
//!
//! Caller: `sub_45CE32`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::sound_bands::Twin;

pub const TWIN: Twin = (0x45D0F1, 0x6D3CD0, 0x6D1AE8, 10);
