//! `system::restore_window` (0x405A33): the game's window restored if it is minimised
//! (`IsIconic`, then `ShowWindow` with `SW_RESTORE`), every register preserved. Not hooked
//! (two Win32 calls, section 10); `audio::set_mode`'s Rust version calls this.

use crate::session::MACHINE;
use windows_sys::Win32::UI::WindowsAndMessaging::{IsIconic, SW_RESTORE, ShowWindow};

pub fn restore_window() {
    unsafe {
        let window = crate::pointers::host_at(MACHINE.window.get() as usize) as _;
        if IsIconic(window) != 0 {
            ShowWindow(window, SW_RESTORE);
        }
    }
}
