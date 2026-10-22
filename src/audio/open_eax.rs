//! `audio::open_eax` (0x40D27F): EAX 1's reverb opened through a 3D buffer.
//!
//! In:  ebx = where the 3D buffer is (a voice's `buffer_3d`)
//! Out: carry = not opened; every register preserved
//!
//! See `audio/property_sets.rs`: nothing with `g_ignoreEaxSetting` or the set already there
//! (`g_eaxProperties`); "AUD: Using EAX" logged, `g_hasEax` 1. One caller,
//! `audio::make_buffer` with bit 0x1000000, where ebx is not always the holder (the game's
//! bug, `notes/bugs.md`: refused when no pointer). Shadow mode runs the original alone.
//! Self-test: a redirected copy whose three variables are the test's and whose log call
//! notes eax, 9 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::property_sets::*;
use crate::errors::Refused;

/// True for the carry clear.
pub unsafe fn open_eax(holder: u32) -> Result<bool, Refused> {
    unsafe { open_property_set(&Opening::eax(), holder, &mut game_log) }
}
