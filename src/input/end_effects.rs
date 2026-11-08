//! `input::end_effects` (0x40F2B8): the race's force feedback ended: both records'
//! effects asked to stop (`input::stop_effects`).
//!
//! See `input/effects.rs`. One caller, `input::device_command` (command 2, the race's
//! end).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::owner::Input;

/// Both devices' effects stopped.
pub fn end_effects(input: &Input) -> Result<(), Refused> {
    crate::input::stop_effects::stop_effects(input)
}
