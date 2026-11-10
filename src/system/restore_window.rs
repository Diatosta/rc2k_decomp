//! `system::restore_window` (0x405A33): the game's window restored if it is minimised (`IsIconic`,
//! then `ShowWindow` with `SW_RESTORE`). Two Win32 calls in the original; `audio::set_mode` and the
//! network's calls use this. The window is SDL's (`platform::restore_window`, done on the window's
//! thread at its next pump).

pub fn restore_window() {
    crate::platform_events::restore_window();
}
