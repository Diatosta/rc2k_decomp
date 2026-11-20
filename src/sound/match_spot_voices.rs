//! `sound::match_spot_voices`: each shared spot given its voice.
//!
//! For each shared spot whose kind is 1 .. 0x15: the voice with its id (the first, searching all
//! 0x48), or else the first free one, is made its: id, new flag 1, the kind's sound, kind, bit,
//! repeats, playing and restart 0. No voice free: the spot goes unheard. Then the voice's place is
//! the spot's and it is heard. See `sound/spot_voices.rs`. Refuses (before anything) more than 36
//! spots (the table's size).

use crate::errors::{BadInput, Refused};
use crate::sound::spot_voices::{MOST_KIND, MOST_SPOTS, load_voices, store_voices};

pub fn match_spot_voices(sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    use rc2k_core::sound::spot_voices::Spot;
    // The logic is `rc2k_core::sound::spot_voices::match_spots`.
    let stage = sound.race.stage.get();
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
                .map(crate::memory::session::PlainFloat::bits),
            kind: spot.kind.get(),
        })
        .collect();
    let kinds: [u32; MOST_KIND as usize + 1] = sound.race.spot_kinds.get().kind_sounds;
    let mut voices = load_voices(sound);
    rc2k_core::sound::spot_voices::match_spots(&spots, &mut voices, &kinds)?;
    store_voices(sound, &voices);
    Ok(())
}
