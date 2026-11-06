//! `audio::release_voice_manager` (0x40C74C): DirectSound's voice manager (its `IKsPropertySet`) released.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! If `g_voiceManager` holds an object, its `Release` (+8), and the variable 0. Callers:
//! `audio::start` (0x40A0D6) and `audio::free_voices`.

use crate::audio::owner::audio;

// ---- Rust version ----

pub fn release_voice_manager() {
    let state = &audio().state;
    let object = state.voice_manager.get();
    if object != 0 {
        crate::platform_sound::release(object);
        state.voice_manager.set(0);
    }
}
