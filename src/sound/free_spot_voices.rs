//! `sound::free_spot_voices` (0x40E6F4): the voices not heard this turn freed.
//!
//! In:  esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! For each voice not heard (+4 not 1): a playing one (+0x28 1) whose id is not free has
//! its sound stopped (`sound::stop_sound`, the request's sound = +0x14) and is freed; a
//! playing free one is left; one not playing is freed. Freed: id 0x1888FFFF, sound -2
//! (+0x28 is left as it is). See `sound/spot_voices.rs`. One caller,
//! `sound::spot_sounds`.
//! Refuses (before anything) a null request.
//! Shadow mode runs the original alone (it stops sounds).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, on the
//! game's voices (put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::sound::spot_voices::{VOICE_COUNT, voice};
use crate::sound::turn_parts::turn_entry;

// ---- Rust version ----

/// Converted: `rc2k_core::sound::spot_voices::free_unheard`; each stop sent with the
/// registers the original has there (the request's sound written first).
pub fn free_spot_voices_in(r: &mut Regs) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, store_voices};
    if r.esi == 0 {
        return Err(BadInput("no request").into());
    }
    let mut voices = load_voices();
    let mut c = *r;
    let done = rc2k_core::sound::spot_voices::free_unheard::<Refused>(&mut voices, |n, sound| {
        (c.ebx, c.ecx, c.eax) = (voice(n as u32), VOICE_COUNT - n as u32, sound);
        unsafe { (c.esi as *mut u32).write_unaligned(sound) };
        crate::sound::turn_parts::stop(&c)
    });
    store_voices(&voices);
    done
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, free_spot_voices_in)
}
