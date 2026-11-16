//! One car's count down: for a car not finished and running, the time since the count down's
//! start (nothing before it) held at 1500, in seconds (/ 100), -1 from 6 on, is stored where the
//! caller asks; on a whole second not -1 the second's sound is cued (`drive::countdown_cue`).
//! Refuses what `countdown_cue` refuses.

use crate::errors::Refused;
use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use rc2k_core::drive::countdown::{self, Beep};

/// The car block `block`, the second into `to`.
pub fn countdown_beep(
    race: &Race,
    block: &Car,
    to: &PlainDword,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let beep = countdown::countdown_beep(
        block.false_start_count.get(),
        block.driver_kind.get(),
        || {
            block.run_ticks.get().wrapping_sub(
                race.frame
                    .panel_and_overlay
                    .get()
                    .overlay
                    .countdown_start
                    .get(),
            )
        },
    );
    let (second, cue) = match beep {
        Beep::Idle | Beep::Before(_) => return Ok(()),
        Beep::At { second, cue, .. } => (second, cue),
    };
    to.set(second);
    if cue {
        // The cue is only for 0 .. SECONDS.
        let sound =
            race.frame.panel_and_overlay.get().overlay.countdown_sounds[second as usize].get();
        race.shared.physics_tables.get().countdown_sound.set(sound);
        crate::drive::countdown_cue::countdown_cue(race, sound, audio)?;
    }
    Ok(())
}
