/// Sets the music volume.
use crate::sound::senders::*;
use rc2k_core::audio::commands::{Command, StreamAction};

/// Nothing if the volume is the one in use.
pub fn set_music_volume(volume: u32, o: crate::owners::Owners<'static>) {
    // The logic is `rc2k_core::sound::senders::new_music_volume`.
    if rc2k_core::sound::senders::new_music_volume(volume, o.audio.state.music_stream.get().volume)
    {
        send_and_wait(
            o,
            Command::Music {
                tune: 0,
                next: 0,
                volume,
                action: StreamAction::Volume,
            },
        );
    }
}
