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
//! Caller: `drive::countdown_beeps`.
//! Refuses what `countdown_cue` refuses.
//! Self-test: the original and the Rust version on a test block and the game's variables
//! (saved and put back), the car with no sound slot: before, during, after.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX};
use crate::physics::flow::{g, p};
use crate::screens::harness::Left;
use rc2k_core::drive::countdown::{self, Beep};

/// The Rust version: the block at `block`, the second into `to`.
// Converted: `rc2k_core::drive::countdown::countdown_beep`.
pub unsafe fn countdown_beep(block: u32, to: u32) -> Result<Left, Refused> {
    let rd = |o: u32| unsafe { ((block + o) as *const u32).read_unaligned() };
    let beep =
        countdown::countdown_beep(rd(0x1C), rd(0x25C), || rd(0x14).wrapping_sub(g(0x6D1814)));
    let (second, remainder, cue) = match beep {
        Beep::Idle => return Ok(Left::nothing()),
        Beep::Before(t) => return Ok(Left::nothing().with(EAX, t)),
        Beep::At {
            second,
            remainder,
            cue,
        } => (second, remainder, cue),
    };
    unsafe { (to as *mut u32).write_unaligned(second) };
    let mut left = Left::nothing()
        .with(EAX, second)
        .with(EDX, remainder)
        .with(EBX, 100);
    if cue {
        let sound = g(0x6D1818 + 4 * second as usize);
        p(0x614114, sound);
        left = left
            .with(EBX, sound)
            .with(EAX, crate::drive::countdown_cue::countdown_cue(sound)?);
    }
    Ok(left)
}
