//! `sound::free_spot_voices`: the voices not heard this turn freed.
//!
//! Takes the request. For each voice not heard: a playing one whose id is not free has its sound
//! stopped (`sound::stop_sound`) and is freed; a playing free one is left; one not playing is
//! freed. Freed: id 0x1888FFFF, sound -2 (the playing flag is left as it is). See
//! `sound/spot_voices.rs`.

use crate::errors::Refused;
use crate::sound::sets_store::SoundRequest;

/// The logic is `rc2k_core::sound::spot_voices::free_unheard`; each stop through `request`
/// (its sound written first).
pub fn free_spot_voices(
    request: &SoundRequest,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, store_voices};
    let mut voices = load_voices(sound);
    let done = rc2k_core::sound::spot_voices::free_unheard::<Refused>(&mut voices, |_, sound| {
        request.sound.set(sound);
        crate::sound::turn_parts::stop(request, audio)
    });
    store_voices(sound, &voices);
    done
}
