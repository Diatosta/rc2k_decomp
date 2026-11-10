//! `system::restore_error_mode` (0x402B95): at the game's end, if the error mode was
//! changed (`dword_4E059C` 1, `system::set_error_mode`, agent A's), the switch 0 and the mode it
//! had before (`uMode`) put back (`SetErrorMode`).
//!
//! One caller, `start`'s way out (0x401116, also the short one), which calls it directly.
pub fn restore_error_mode(system: &crate::system::owner::System) {
    if system.start.error_mode.changed.get() != 1 {
        return;
    }
    system.start.error_mode.changed.set(0);
    crate::os::set_error_mode(system.start.error_mode.mode.get());
}
