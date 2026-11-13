//! `audio::release_eax` (0x40D2F0): the EAX property set released.
//!
//! If `g_eaxProperties` holds an object, its `Release` (+8), the variable 0, and the room
//! asked for and the room set (`g_eaxRoom`, `g_eaxRoomSet`) both 6. Callers: `audio::start`
//! (0x40A0D1) and `audio::free_voices`.

use crate::audio::own::AudioOwn;

/// The property set released, and the room asked for and the room set back to 6.
pub fn release_eax(state: &AudioOwn) {
    let object = state.eax_properties.get();
    if object != 0 {
        crate::platform_sound::release(object);
        state.eax_properties.set(0);
        state.eax_room.set(6);
        state.eax_room_set.set(6);
    }
}
