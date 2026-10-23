//! `system::restore_error_mode` (0x402B95): at the game's end, if the error mode was
//! changed (`dword_4E059C` 1, `sub_402B7C`, agent A's), the switch 0 and the mode it had
//! before (`uMode`) put back (`SetErrorMode`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out (0x401116, also the short one), which calls it directly
//! (2026-10-23); the registers above are the original's, for reference.

use crate::framework::{game_read, game_write, rva};

const CHANGED_RVA: usize = rva(0x4E059C); // dword_4E059C: 1 after `sub_402B7C` set the mode
const MODE_RVA: usize = rva(0x4E05A0); // uMode: the mode before

/// The Rust version (see the header).
pub fn restore_error_mode() {
    use windows_sys::Win32::System::Diagnostics::Debug::{SetErrorMode, THREAD_ERROR_MODE};
    unsafe {
        if game_read::<u32>(CHANGED_RVA) != 1 {
            return;
        }
        game_write(CHANGED_RVA, 0u32);
        SetErrorMode(game_read::<THREAD_ERROR_MODE>(MODE_RVA));
    }
}
