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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};

// ---- Rust version ----

/// Converted: `rc2k_core::sound::spot_voices::free_unheard`; each stop through `request`
/// (its sound written first).
pub fn free_spot_voices(request: u32) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, store_voices};
    if request == 0 {
        return Err(BadInput("no request").into());
    }
    let mut voices = load_voices();
    let done = rc2k_core::sound::spot_voices::free_unheard::<Refused>(&mut voices, |_, sound| {
        crate::sound::turn_parts::put(request, 0, sound)?;
        crate::sound::turn_parts::stop(request)
    });
    store_voices(&voices);
    done
}
