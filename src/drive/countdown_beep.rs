//! `drive::countdown_beep` (0x440EA3): one car's count down: for a car not finished
//! (`false_start_count` 0) and running (`driver_kind`), the time since the count down's start
//! (`run_ticks` less `g_countdownStart`; nothing before it) held at 1500, in seconds (/ 100), -1
//! from 6 on, is stored where the caller asks; on a whole second not -1 the second's sound
//! (`countdown_sounds[s]`, remembered in `countdown_sound`) is cued (`drive::countdown_cue`).
//!
//! Takes: the car's block, where its second goes
//!
//! Caller: `drive::countdown_beeps`. Refuses what `countdown_cue` refuses.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::session::PlainLoose;
use rc2k_core::drive::countdown::{self, Beep};

/// The car block `block`, the second into `to`.
// Converted: `rc2k_core::drive::countdown::countdown_beep`.
pub fn countdown_beep(
    race: &Race,
    block: &Car,
    to: &PlainLoose,
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
        let sound =
            race.frame.panel_and_overlay.get().overlay.countdown_sounds[second as usize].get(); // dword_6D1818, the cue is only for 0 .. SECONDS
        race.shared.physics_tables.get().countdown_sound.set(sound);
        crate::drive::countdown_cue::countdown_cue(race, sound, audio)?;
    }
    Ok(())
}
