//! `audio::release_eax` (0x40D2F0): the EAX property set released.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! If `g_eaxProperties` holds an object, its `Release` (+8), the variable 0, and the room
//! asked for and the room set (`g_eaxRoom`, `g_eaxRoomSet`) both 6. Callers: `audio::start`
//! (0x40A0D1) and `audio::free_voices`.

use crate::audio::com::{self, RELEASE};
use crate::audio::store::AUDIO;

// ---- Rust version ----

/// The property set released, and the room asked for and the room set back to 6.
pub fn release_eax() {
    let object = AUDIO.eax_properties.get();
    if object != 0 {
        unsafe { com::call(object, RELEASE) };
        AUDIO.eax_properties.set(0);
        AUDIO.eax_room.set(6);
        AUDIO.eax_room_set.set(6);
    }
}
