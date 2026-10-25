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
use crate::race::viewer_screen_store::VIEWER_PANEL;

/// The Rust version. `eax` the ms a picture, or 0 for the rate's.
pub fn start_frame_pictures(eax: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer::{picture_rate, picture_time}`.
    use rc2k_core::race::viewer::{picture_rate, picture_time};
    let rate = picture_rate(VIEWER_PANEL.picture_rate());
    if VIEWER_PANEL.picture_rate() != rate {
        VIEWER_PANEL.set_picture_rate(rate);
    }
    VIEWER_PANEL.set_picture_time(picture_time(eax, rate));
    // The folder made.
    unsafe {
        windows_sys::Win32::Storage::FileSystem::CreateDirectoryA(
            (VIEWER_PANEL.picture_folder.addr() as u32) as *const u8,
            std::ptr::null(),
        )
    };
    VIEWER_PANEL.replay_recording.set(1);
    VIEWER_PANEL.pictures_started.set(0);
    VIEWER_PANEL.picture_clock.set(0);
    Ok(())
}
