//! `system::restore_error_mode` (0x402B95): at the game's end, if the error mode was
//! changed (`dword_4E059C` 1, `sub_402B7C`, agent A's), the switch 0 and the mode it had
//! before (`uMode`) put back (`SetErrorMode`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out (0x401116, also the short one), which calls it directly
//! (2026-10-23); the registers above are the original's, for reference.

use crate::system::set_error_mode::ERROR_MODE;

/// The Rust version (see the header).
pub fn restore_error_mode() {
    if ERROR_MODE.changed.get() != 1 {
        return;
    }
    ERROR_MODE.changed.set(0);
    crate::os::set_error_mode(ERROR_MODE.mode.get());
}
