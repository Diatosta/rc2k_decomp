//! `race::start_frame_pictures` (0x41F35E): the replay's frames to be saved as pictures
//! (`race::replay_screenshot` does it each frame): the rate `dword_5C2CEF` held to 1 ..
//! 100 (signed); the time a picture (`dword_5C2CF3`) eax, or for eax 0 1000 / the rate;
//! the folder `rpl` made (`sub_403113`: `CreateDirectoryA`, edx the name);
//! `g_replayRecording` 1, `dword_5C2CD4` and `dword_5C2CD0` 0.
//!
//! In:  eax the ms a picture, or 0 for the rate's
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_action` (actions 0x25 .. 0x28: eax 0, 0xA0, 0x50, 0x28). Its call
//! through `part` (1 the folder made; in the game `CreateDirectoryA`). Shadow mode runs
//! the original alone (it makes a folder). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version: rates below, inside and past the range, eax 0 or
//! not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::a;
use crate::race::viewer::{
    PICTURE_RATE, PICTURE_TIME, PICTURES_STARTED, PICTURES_STARTED_2, RECORDING, get, put,
};

const FOLDER: usize = 0x5C2CD8; // "rpl"

/// The Rust version.
pub fn start_frame_pictures(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer::{picture_rate, picture_time}`.
    use rc2k_core::race::viewer::{picture_rate, picture_time};
    let rate = picture_rate(get(PICTURE_RATE));
    if get(PICTURE_RATE) != rate {
        put(PICTURE_RATE, rate);
    }
    let mut r = *entry;
    r.eax = picture_time(r.eax, rate);
    put(PICTURE_TIME, r.eax);
    r.edx = a(FOLDER);
    crate::race::start_frame_pictures::game_part(1, &mut r)?;
    put(RECORDING, 1);
    put(PICTURES_STARTED, 0);
    put(PICTURES_STARTED_2, 0);
    Ok(*entry)
}

/// In the game: the folder made.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        windows_sys::Win32::Storage::FileSystem::CreateDirectoryA(
            r.edx as *const u8,
            std::ptr::null(),
        )
    };
    Ok(())
}
