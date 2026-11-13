//! Begins the race's force feedback: when a race is on (any sound object in use) both records'
//! effects are asked to start and the clock is kept as the effects' last step. See
//! `input/effects.rs`.

use crate::errors::Refused;
use crate::input::owner::Input;

/// The effects started with sound objects in use, and the
/// effects' clock set.
pub fn begin_effects(input: &Input, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    if sound.race.shared.count.is_set() {
        crate::input::start_effects::start_effects(input)?;
    }
    input
        .force_feedback
        .last_time
        .set(crate::system::tick_count::tick_count());
    Ok(())
}
