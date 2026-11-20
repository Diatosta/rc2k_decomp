//! `sound::merge_spot_voices`: of the voices heard with one sound, the nearest kept and
//! (re)started, moved to the point between it and the next nearest that is nearest the listener;
//! all the others silenced.
//!
//! Takes the sound. The pickers' work (`SpotPicks.pick_playing` .. `SpotPicks.pick_next`) is
//! cleared (distances to -1). For each voice heard with the sound: a playing one is noted in
//! `SpotPicks.pick_playing`; its distance x 1000 as a whole number is compared (unsigned) with the
//! nearest's and the next's, which move down; the voice is silenced. The nearest, if any, is
//! started; with a next as well, `math::nearest_on_segment` of their places and the listener (0, 0,
//! 0) gives the nearest's new place.

/// The logic is `rc2k_core::sound::spot_voices::merge` and `merge_between`. The voices of `sound`
/// merged.
pub(super) fn merge_spot_voices(
    sound: u32,
    owner: &crate::sound::owner::Sound,
    system: &crate::system::owner::System,
) {
    use crate::sound::spot_voices::{load_voices, store_picks, store_voices};
    use rc2k_core::sound::spot_voices as core;
    let mut voices = load_voices(owner);
    let (picks, trail) = core::merge(&mut voices, sound);
    store_voices(owner, &voices);
    store_picks(owner, &picks, &trail, system);
    let (Some(nearest), Some(next)) = (picks.nearest, picks.next) else {
        return;
    };
    let (a, b) = (voices[nearest].place, voices[next].place);
    // The point on the segment between the two places that is nearest the listener (0, 0, 0).
    let point = crate::math::nearest_on_segment::nearest_on_segment(a, b, [0; 3]);
    core::merge_between(&mut voices, &picks, point);
    store_voices(owner, &voices);
}
