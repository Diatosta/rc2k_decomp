//! `input::start_effects` (0x409659): both device records' effects asked to start
//! (`input::start_device_effects` on `g_inputRecord1`, then `g_inputRecord2`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. One caller, `input::begin_effects`. Its calls through `part`.
//! Shadow mode runs the original alone. Self-test: a copy whose calls go to the noting
//! stand-ins, against the Rust version (`stop_effects` shares it).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::effects::{RECORD_1, RECORD_2};

/// Both device records.
pub fn records() -> [u32; 2] {
    [RECORD_1, RECORD_2].map(|record| record.address() as u32)
}

/// Both devices' effects started.
pub fn start_effects() -> Result<(), Refused> {
    for record in records() {
        crate::input::start_device_effects::start_device_effects(record)?;
    }
    Ok(())
}
