//! `sound::match_spot_voices` (0x40E642): each shared spot given its voice.
//!
//! For each of the `g_sharedSoundSpotCount` spots whose kind is 1 .. 0x15: the voice with
//! its id (the first, searching all 0x48), or else the first free one, made its: id,
//! new (+0x20) 1, the kind's sound, kind, bit, repeats, playing and restart 0. No voice
//! free: the spot goes unheard. Then the voice's place = the spot's and heard (+4) 1.
//! See `sound/spot_voices.rs`. One caller, `sound::spot_sounds`.
//! Refuses (before anything) more than 36 spots (the table's size).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::sound::spot_voices::{MOST_KIND, MOST_SPOTS, load_voices, store_voices};

// ---- Rust version ----

pub fn match_spot_voices(sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    use rc2k_core::sound::spot_voices::Spot;
    // Converted: `rc2k_core::sound::spot_voices::match_spots`.
    let stage = &sound.race.stage;
    let count = stage.shared_count.get();
    if count > MOST_SPOTS {
        return Err(BadInput("more than 36 shared spots").into());
    }
    let spots: Vec<Spot> = stage.shared_spots[..count as usize]
        .iter()
        .map(|spot| Spot {
            id: spot.id.get(),
            place: spot
                .place
                .each_ref()
                .map(super::super::session::Float::bits),
            kind: spot.kind.get(),
        })
        .collect();
    let kinds: [u32; MOST_KIND as usize + 1] = sound
        .race
        .spot_picks
        .kind_sounds
        .each_ref()
        .map(super::super::session::Dword::get);
    let mut voices = load_voices(sound);
    rc2k_core::sound::spot_voices::match_spots(&spots, &mut voices, &kinds)?;
    store_voices(sound, &voices);
    Ok(())
}
