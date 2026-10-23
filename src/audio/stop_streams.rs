//! `audio::stop_streams` (0x40C5E8): both stream commands with eax 0.
//!
//! In:  ebx, ecx, edx = passed on (callers: the thread's end 0x409DD6, the mode change
//!      0x40A43C)
//! Out: every register preserved
//!
//! `audio::stream_command` and then `audio::music_command`, each with eax 0 and the
//! caller's ebx, ecx and edx: with action 0 (or any but 1 and 3) both stop. Shadow mode
//! runs the original alone. Self-test: a redirected copy whose two calls go to stand-ins
//! noting their registers, 4 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//!
//! Plain since 2026-10-23 (phase 3): action 0 (both callers now stop: section 16), with
//! which neither command reads its other arguments.

use crate::audio::stream::SOUND_STREAM_RVA;
use crate::audio::stream_commands::{MusicVars, music_command, stream_command};
use crate::errors::Refused;
use crate::framework::game_addr;

/// Both streams stopped.
pub fn stop_streams() -> Result<(), Refused> {
    stream_command(0, 0, 0, game_addr(SOUND_STREAM_RVA) as u32)?;
    music_command(0, 0, 0, 0, &MusicVars::game())
}
