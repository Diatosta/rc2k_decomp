//! Opens DirectSound's voice manager ("DSV") through a 3D buffer (see `audio/property_sets.rs`):
//! nothing when the DSV setting is ignored or it is already there; the mode is set to 2 and
//! "AUD: Using DSV" logged.

use crate::audio::own::AudioOwn;
use crate::audio::property_sets::*;
use crate::errors::Refused;

/// True when it worked.
pub fn open_voice_manager(state: &AudioOwn, holder: Holder) -> Result<bool, Refused> {
    open_property_set(state, &Opening::voice_manager(state), holder)
}
