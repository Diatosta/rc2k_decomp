//! Both stream commands with action 0: the stream and the music stopped.

use crate::audio::own::AudioThread;
use crate::audio::stream_commands::{music_command, stream_command};
use crate::errors::Refused;

/// Both streams stopped.
pub fn stop_streams(audio: &AudioThread) -> Result<(), Refused> {
    stream_command(audio, 0, 0, 0)?;
    music_command(&audio.state, 0, 0, 0, 0)
}
