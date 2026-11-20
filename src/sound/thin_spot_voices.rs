//! `sound::thin_spot_voices`: of the voices heard with one sound, the nearest two found and one of
//! them, by a random bit, kept heard; unless one is playing.
//!
//! Takes the sound. The pickers' work is cleared. For each voice heard with the sound: one already
//! playing ends it at once (nothing more changes). Else its distance x 1000 against the nearest's
//! and the next's (unsigned): a new nearest moves the old one to next and stays heard; a new next,
//! and every other, is silenced. A nearest displaced stays heard (a slip kept as is). With a next
//! found: `math::next_random` & 1 for the nearest's heard flag, the other bit for the next's.

/// The logic is `rc2k_core::sound::spot_voices::thin` and `thin_choose`. The voices of `sound`
/// thinned.
pub(super) fn thin_spot_voices(
    sound: u32,
    owner: &crate::sound::owner::Sound,
    system: &crate::system::owner::System,
    random: &crate::math::next_random::Randoms,
) {
    use crate::sound::spot_voices::{load_voices, store_picks, store_voices};
    use rc2k_core::sound::spot_voices as core;
    let mut voices = load_voices(owner);
    let (picks, trail) = core::thin(&mut voices, sound);
    store_voices(owner, &voices);
    store_picks(owner, &picks, &trail, system);
    if trail.ended_at.is_some() {
        return;
    }
    let (Some(_), Some(_)) = (picks.nearest, picks.next) else {
        return;
    };
    let drawn = random.sounds.next();
    core::thin_choose(&mut voices, &picks, drawn);
    store_voices(owner, &voices);
}
