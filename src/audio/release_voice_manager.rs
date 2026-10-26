//! `audio::release_voice_manager` (0x40C74C): DirectSound's voice manager (its `IKsPropertySet`) released.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! If `g_voiceManager` holds an object, its `Release` (+8), and the variable 0. Callers:
//! `audio::start` (0x40A0D6) and `audio::free_voices`.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose variable is the
//! test's, with and without a test object.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

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
