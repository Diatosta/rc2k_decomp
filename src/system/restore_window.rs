//! `system::restore_window` (0x405A33): the game's window restored if it is minimised
//! (`IsIconic`, then `ShowWindow` with `SW_RESTORE`). Not a hook
//! (two Win32 calls, section 10); `audio::set_mode`'s Rust version and the network's calls
//! call this. Since phase 11 the window is SDL's (`platform::restore_window`, done on the
//! window's thread at its next pump): before, the game's `hWnd` was 0 in the standalone and
//! nothing was restored.

pub fn restore_window() {
    crate::platform_events::restore_window();
}
