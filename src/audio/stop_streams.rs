//! `audio::stop_streams` (0x40C5E8): both stream commands with eax 0.
//!
//! In:  ebx, ecx, edx = passed on (callers: the thread's end 0x409DD6, the mode change
//!      0x40A43C)
//!
//! `audio::stream_command` and then `audio::music_command`, each with eax 0 and the
//! caller's ebx, ecx and edx: with action 0 (or any but 1 and 3) both stop.
//!
//! Plain since 2026-10-23 (phase 3): action 0 (both callers now stop: section 16), with
//! which neither command reads its other arguments.

use crate::audio::owner::Audio;
use crate::audio::stream_commands::{music_command, stream_command};
use crate::errors::Refused;

/// Both streams stopped.
pub fn stop_streams(audio: &Audio) -> Result<(), Refused> {
    stream_command(audio, 0, 0, 0)?;
    music_command(&audio.state, 0, 0, 0, 0)
}
