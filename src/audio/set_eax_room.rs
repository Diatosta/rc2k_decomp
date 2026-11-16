//! Sets the EAX room: returns the room's entry in the rooms table. Refuses a room past the table's
//! ten.

use crate::audio::own::AudioOwn;
use crate::audio::property_sets::set_eax_room;
use crate::errors::Refused;

/// On the game's variables.
pub fn in_game(state: &AudioOwn) -> Result<(), Refused> {
    set_eax_room(state, state.eax_room.get())
}
