//! `race::play_replay_range` (0x41D846): the viewer plays from the block read: its
//! cameras into the block (`drive::replay_frame_marks`), the race's sounds started unless
//! the frames are being saved as pictures (`race::start_race_sounds`, not with
//! `g_replayRecording` 1), the replay played (`race::play_movie`), the sounds reset
//! (`race::reset_race_sounds`), the key that ended it (`input::take_key`) into
//! `dword_5C2550`, `dword_5C2554` 1 (keys held), the block shown held to the last
//! (`dword_5BECD0` at most, unsigned; then frame 0), `dword_5BEC98` 1 (stopped).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::play_marked`. Its calls through `part`: 1 `replay_frame_marks`, 2
//! `start_race_sounds`, 3 `play_movie`, 4 `reset_race_sounds`, 5 `input::take_key`; in the
//! game their Rust versions. Ungated (it plays a replay). Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins, against the Rust version:
//! recording or not, the block before, at and past the last.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::race::viewer::{
    BLOCK, BLOCKS, FRAME, KEY_AFTER_PLAY, RECORDING, STOPPED, WAIT_RELEASE, get, put,
};

/// The Rust version (every register kept).
pub fn play_replay_range() -> Result<(), Refused> {
    use crate::race::race_end;
    crate::drive::replay_frame_marks::replay_frame_marks();
    if get(RECORDING) != 1 {
        crate::race::sounds::start_race_sounds()?;
    }
    crate::race::play_movie::play_movie()?;
    race_end::reset_race_sounds(&race_end::SoundVars::game())?;
    // The key taken: al, the rest of eax cleared (`and eax, 0FFh`).
    let key = crate::input::take_key::take_key() as u32 & 0xFF;
    put(KEY_AFTER_PLAY, key);
    put(WAIT_RELEASE, 1);
    if get(BLOCKS) <= get(BLOCK) {
        put(BLOCK, get(BLOCKS));
        put(FRAME, 0);
    }
    put(STOPPED, 1);
    Ok(())
}
