//! Opens EAX through a voice's 3D buffer (see `audio/property_sets.rs`); nothing when the EAX
//! setting is ignored or the set is already there.

use crate::audio::own::AudioOwn;
use crate::audio::property_sets::*;
use crate::errors::Refused;

/// True when it worked.
pub(super) fn open_eax(state: &AudioOwn, holder: Holder) -> Result<bool, Refused> {
    open_property_set(state, &Opening::eax(state), holder)
}
