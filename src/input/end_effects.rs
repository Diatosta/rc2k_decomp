//! Ends the race's force feedback (command 2, the race's end): both records' effects asked to
//! stop (`input::stop_effects`). See `input/effects.rs`.

use crate::errors::Refused;
use crate::input::owner::Input;

/// Both devices' effects stopped.
pub fn end_effects(input: &Input) -> Result<(), Refused> {
    crate::input::stop_effects::stop_effects(input)
}
