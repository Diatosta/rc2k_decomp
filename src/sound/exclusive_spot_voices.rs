//! `sound::exclusive_spot_voices`: at most one voice of a kind starts: a kind already playing, or
//! started by an earlier voice in the table, silences the others.
//!
//! The kinds' bits of the voices heard and playing are gathered; then each voice heard and not
//! playing whose bit is among them is silenced, and its bit added either way. One caller,
//! `sound::spot_sounds`.

use crate::sound::spot_voices::{load_voices, store_voices};

pub fn exclusive_spot_voices(sound: &crate::sound::owner::Sound) {
    // The logic is `rc2k_core::sound::spot_voices::keep_exclusive`.
    let mut voices = load_voices(sound);
    rc2k_core::sound::spot_voices::keep_exclusive(&mut voices);
    store_voices(sound, &voices);
}
