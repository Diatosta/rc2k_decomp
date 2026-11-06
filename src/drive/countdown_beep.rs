//! `drive::countdown_beep` (0x440EA3): one car's count down (the block at esi, its figure
//! at edi): for a car not finished (+0x1C 0) and running (+0x25C), the time since the
//! count down's start (+0x14 less `g_countdownStart`; nothing before it) held at 1500,
//! in seconds (/ 100), -1 from 6 on, into [edi]; on a whole second not -1 the second's
//! sound (`dword_6D1818[s]`, into `dword_614114`) cued (`drive::countdown_cue`).
//!
//! In:  esi the car's block, edi where its second goes
//! Out: eax the second, edx the remainder, ebx 100 or the sound; the rest kept; no flags
//!      read
//!
//! Caller: `drive::countdown_beeps`. Refuses what `countdown_cue` refuses.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::race::panel_store::Loose32;
use rc2k_core::drive::countdown::{self, Beep};

/// The Rust version: the car block `block`, the second into `to`.
// Converted: `rc2k_core::drive::countdown::countdown_beep`.
pub fn countdown_beep(race: &Race, block: &Car, to: &Loose32) -> Result<(), Refused> {
    let beep = countdown::countdown_beep(
        block.false_start_count.bits(),
        block.driver_kind.bits(),
        || {
            block
                .run_ticks
                .bits()
                .wrapping_sub(race.frame.panel_and_overlay.overlay.countdown_start.get())
        },
    );
    let (second, cue) = match beep {
        Beep::Idle | Beep::Before(_) => return Ok(()),
        Beep::At { second, cue, .. } => (second, cue),
    };
    to.set(second);
    if cue {
        let sound = race.frame.panel_and_overlay.overlay.countdown_sounds[second as usize].get(); // dword_6D1818, the cue is only for 0 .. SECONDS
        race.shared.physics_tables.countdown_sound.set(sound);
        crate::drive::countdown_cue::countdown_cue(race, sound)?;
    }
    Ok(())
}
