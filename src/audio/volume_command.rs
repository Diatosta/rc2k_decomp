//! Command 8: the master volume. Nothing for a negative volume; else it is kept and
//! `audio::set_master_volume` called with it.

use crate::audio::own::AudioOwn;

/// `set` sets the master volume.
pub fn volume_command(state: &AudioOwn, volume: u32, set: &mut dyn FnMut(u32)) {
    let Some(volume) = rc2k_core::audio::levels::kept_master_volume(volume) else {
        return;
    };
    state.master_volume.set(volume);
    set(volume);
}
