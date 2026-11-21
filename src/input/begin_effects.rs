//! Begins the race's force feedback: when a race is on (any sound object in use) both records'
//! effects are asked to start and the clock is kept as the effects' last step. See
//! `input/effects.rs`.

use crate::errors::Refused;
use crate::input::owner::Input;

/// The effects started with sound objects in use, and the
/// effects' clock set.
pub(super) fn begin_effects(
    input: &Input,
    sound: &crate::sound::store::SoundShared,
    picture_clock: &crate::clock::tick_count::PictureClock,
) -> Result<(), Refused> {
    if sound.count.is_set() {
        crate::input::start_effects::start_effects(input)?;
    }
    input
        .force_feedback
        .last_time
        .set(crate::clock::tick_count::tick_count(picture_clock));
    Ok(())
}
