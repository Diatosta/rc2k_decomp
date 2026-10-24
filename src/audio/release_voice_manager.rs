//! `audio::release_voice_manager` (0x40C74C): DirectSound's voice manager (its `IKsPropertySet`) released.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! If `g_voiceManager` holds an object, its `Release` (+8), and the variable 0. Callers:
//! `audio::start` (0x40A0D6) and `audio::free_voices`.

use crate::audio::com::{self, RELEASE};
use crate::audio::voices::*;

// ---- Rust version ----

pub unsafe fn release_voice_manager(at: u32) {
    unsafe {
        let object = read(at);
        if object != 0 {
            com::call(object, RELEASE);
            write(at, 0);
        }
    }
}
