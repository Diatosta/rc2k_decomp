//! Releases DirectSound's voice manager (its property set) if there is one.

use crate::audio::own::AudioOwn;

pub(super) fn release_voice_manager(state: &AudioOwn) {
    let object = state.voice_manager.get();
    if object != 0 {
        crate::platform::sound::release(object);
        state.voice_manager.set(0);
    }
}
