//! `audio::stop_streams` (0x40C5E8): both stream commands with action 0.
//!
//! `audio::stream_command` and then `audio::music_command`, each with action 0
//! (any action but 1 and 3 stops the stream).

use crate::audio::own::AudioThread;
use crate::audio::stream_commands::{music_command, stream_command};
use crate::errors::Refused;

/// Both streams stopped.
pub fn stop_streams(audio: &AudioThread) -> Result<(), Refused> {
    stream_command(audio, 0, 0, 0)?;
    music_command(&audio.state, 0, 0, 0, 0)
}
