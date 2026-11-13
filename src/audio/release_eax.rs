//! Releases the EAX property set if there is one, and sets the room asked for and the room set
//! back to 6.

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
