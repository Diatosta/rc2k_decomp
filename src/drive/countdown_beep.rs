//! One car's count down: for a car not finished and running, the time since the count down's
//! start (nothing before it) held at 1500, in seconds (/ 100), -1 from 6 on, is stored where the
//! caller asks; on a whole second not -1 the second's sound is cued (`drive::countdown_cue`).
//! Refuses what `countdown_cue` refuses.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use rc2k_core::drive::countdown::{self, Beep};

/// The car block `block`, the second handed to `store`.
pub fn countdown_beep(
    race: &Race,
    block: &Car,
    store: impl FnOnce(u32),
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let beep = countdown::countdown_beep(block.false_start_count, block.driver_kind, || {
        block
            .run_ticks
            .wrapping_sub(crate::race::overlay_store::COUNTDOWN_START)
    });
    let (second, cue) = match beep {
        Beep::Idle | Beep::Before(_) => return Ok(()),
        Beep::At { second, cue, .. } => (second, cue),
    };
    store(second);
    if cue {
        // The cue is only for 0 .. SECONDS.
        let sound = crate::race::overlay_store::COUNTDOWN_SOUNDS[second as usize];
        crate::drive::countdown_cue::countdown_cue(race, sound, audio)?;
    }
    Ok(())
}
