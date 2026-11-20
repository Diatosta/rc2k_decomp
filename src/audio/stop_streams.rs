//! Both stream commands with action 0: the stream and the music stopped.

use crate::audio::own::AudioThread;
use crate::audio::stream_commands::{music_command, stream_command};
use crate::errors::Refused;
use rc2k_core::audio::commands::StreamAction;

/// Both streams stopped.
pub(super) fn stop_streams(audio: &AudioThread) -> Result<(), Refused> {
    stream_command(audio, 0, 0, StreamAction::Stop)?;
    music_command(&audio.state, 0, 0, 0, StreamAction::Stop)
}
