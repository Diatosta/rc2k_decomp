//! `race::start_frame_pictures` (0x41F35E): the replay's frames to be saved as pictures
//! (`race::replay_screenshot` does it each frame): the rate `dword_5C2CEF` held to 1 ..
//! 100 (signed); the time a picture (`dword_5C2CF3`) eax, or for eax 0 1000 / the rate;
//! the folder `rpl` made (`sub_403113`: `CreateDirectoryA`, edx the name);
//! `g_replayRecording` 1, `dword_5C2CD4` and `dword_5C2CD0` 0.
//!
//! In:  eax the ms a picture, or 0 for the rate's
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_action` (actions 0x25 .. 0x28: eax 0, 0xA0, 0x50, 0x28).

use crate::errors::Refused;
use crate::race::viewer::{
    PICTURE_RATE, PICTURE_TIME, PICTURES_STARTED, PICTURES_STARTED_2, RECORDING, get, put,
};
use crate::race::viewer_screen_store::VIEWER_PANEL;

/// The Rust version. `eax` the ms a picture, or 0 for the rate's.
pub fn start_frame_pictures(eax: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer::{picture_rate, picture_time}`.
    use rc2k_core::race::viewer::{picture_rate, picture_time};
    let rate = picture_rate(get(PICTURE_RATE));
    if get(PICTURE_RATE) != rate {
        put(PICTURE_RATE, rate);
    }
    put(PICTURE_TIME, picture_time(eax, rate));
    // The folder made.
    unsafe {
        windows_sys::Win32::Storage::FileSystem::CreateDirectoryA(
            (VIEWER_PANEL.picture_folder.addr() as u32) as *const u8,
            std::ptr::null(),
        )
    };
    put(RECORDING, 1);
    put(PICTURES_STARTED, 0);
    put(PICTURES_STARTED_2, 0);
    Ok(())
}
