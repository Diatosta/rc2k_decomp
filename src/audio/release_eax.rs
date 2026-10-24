//! `audio::release_eax` (0x40D2F0): the EAX property set released.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! If `g_eaxProperties` holds an object, its `Release` (+8), the variable 0, and the room
//! asked for and the room set (`g_eaxRoom`, `g_eaxRoomSet`) both 6. Callers: `audio::start`
//! (0x40A0D1) and `audio::free_voices`.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose three variables
//! are the test's, with and without a test object.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::com::{self, RELEASE};
use crate::audio::voices::*;
use crate::store::game_addr;

// ---- Rust version ----

/// `vars`: the property set, the room asked for and the room set.
pub unsafe fn release_eax(vars: [u32; 3]) {
    unsafe {
        let object = read(vars[0]);
        if object != 0 {
            com::call(object, RELEASE);
            write(vars[0], 0);
            write(vars[1], 6);
            write(vars[2], 6);
        }
    }
}

pub fn game_vars() -> [u32; 3] {
    [EAX_RVA, EAX_ROOM_RVA, EAX_ROOM_SET_RVA].map(|r| game_addr(r) as u32)
}
