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
use crate::physics::tables_store::PHYSICS;
use crate::race::overlay_store::RACE_OVERLAY;
use rc2k_core::drive::countdown::{self, Beep};

/// The Rust version: the block at `block`, the second into `to`.
// Converted: `rc2k_core::drive::countdown::countdown_beep`.
pub unsafe fn countdown_beep(block: u32, to: u32) -> Result<(), Refused> {
    let rd = |o: u32| unsafe { ((block + o) as *const u32).read_unaligned() };
    let beep = countdown::countdown_beep(rd(0x1C), rd(0x25C), || {
        rd(0x14).wrapping_sub(RACE_OVERLAY.countdown_start.get())
    });
    let (second, cue) = match beep {
        Beep::Idle | Beep::Before(_) => return Ok(()),
        Beep::At { second, cue, .. } => (second, cue),
    };
    unsafe { (to as *mut u32).write_unaligned(second) };
    if cue {
        let sound = RACE_OVERLAY.countdown_sounds[second as usize].get(); // dword_6D1818, the cue is only for 0 .. SECONDS
        PHYSICS.countdown_sound.set(sound);
        crate::drive::countdown_cue::countdown_cue(sound)?;
    }
    Ok(())
}
