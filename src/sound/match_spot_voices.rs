//! `sound::match_spot_voices` (0x40E642): each shared spot given its voice.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! For each of the `g_sharedSoundSpotCount` spots whose kind is 1 .. 0x15: the voice with
//! its id (the first, searching all 0x48), or else the first free one, made its: id,
//! new (+0x20) 1, the kind's sound, kind, bit, repeats, playing and restart 0. No voice
//! free: the spot goes unheard. Then the voice's place = the spot's and heard (+4) 1.
//! See `sound/spot_voices.rs`. One caller, `sound::spot_sounds`.
//! Refuses (before anything) more than 36 spots (the table's size).
//! Shadow mode: both, the voices compared.
//! Self-test: the original against the Rust version on spots and voices of the test's
//! (the game's tables, put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};
use crate::sound::spot_voices::{
    MOST_KIND, MOST_SPOTS, SOUND_OF_KIND, SPOT_COUNT, SPOT_LEN, SPOTS, g, load_voices,
    store_voices, vw,
};
use crate::sound::turn_parts::turn_entry;

// ---- Rust version ----

pub fn match_spot_voices_in(_: &mut Regs) -> Result<(), Refused> {
    use rc2k_core::sound::spot_voices::Spot;
    // Converted: `rc2k_core::sound::spot_voices::match_spots`.
    let count = g(SPOT_COUNT);
    if count > MOST_SPOTS {
        return Err(BadInput("more than 36 shared spots").into());
    }
    let table = game_addr(rva(SPOTS)) as u32;
    let spots: Vec<Spot> = (0..count)
        .map(|s| {
            let spot = table + s * SPOT_LEN;
            Spot {
                id: vw(spot, 0),
                place: [vw(spot, 4), vw(spot, 8), vw(spot, 0xC)],
                kind: vw(spot, 0x10),
            }
        })
        .collect();
    let kinds: [u32; MOST_KIND as usize + 1] =
        unsafe { crate::framework::game_read(rva(SOUND_OF_KIND)) };
    let mut voices = load_voices();
    rc2k_core::sound::spot_voices::match_spots(&spots, &mut voices, &kinds)?;
    store_voices(&voices);
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, match_spot_voices_in)
}
