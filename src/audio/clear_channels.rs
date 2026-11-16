//! Makes the first 0x10 mixer channels free.

use crate::audio::own::AudioThread;
use crate::audio::voices::GamePool;

pub fn clear_channels(audio: &AudioThread) {
    rc2k_core::audio::voices::clear_channels(&mut GamePool::new(audio));
}
