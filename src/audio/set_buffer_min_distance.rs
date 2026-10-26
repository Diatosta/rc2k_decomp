//! `audio::set_buffer_min_distance` (0x40C69A): a voice's 3D buffer's minimum distance.
//!
//! In:  esi = the voice
//! Out: eax = the result; ecx, edx as the method leaves them
//!
//! `SetMinDistance` (+0x44) of its `buffer_3d` with its `min_distance` (an f32),
//! deferred (1). One caller, `audio::make_buffer`. Refuses a voice without its 3D buffer
//! (the original calls through 0). Shadow mode runs the original alone. Self-test: the
//! original on voices whose 3D buffer is the test DirectSound's, 3 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::voices::Voice;
use crate::errors::{BadInput, Refused};

const SET_MIN_DISTANCE: u32 = 0x44;
const DEFERRED: u32 = 1;

pub unsafe fn set_buffer_min_distance(v: &Voice) -> Result<i32, Refused> {
    if v.buffer_3d == 0 {
        return Err(BadInput("a voice without its 3D buffer").into());
    }
    unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(v.buffer_3d, SET_MIN_DISTANCE));
        Ok(f(v.buffer_3d, v.min_distance, DEFERRED))
    }
}
