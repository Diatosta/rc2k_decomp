//! `system::read_keyboard_type` (0x410AA3): `g_keyboardType` (0x518404) 2 when the
//! keyboard's subtype (`GetKeyboardType(1)`) is 0xD01 .. 0xD06 (a Japanese keyboard's);
//! otherwise it is left as it is.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! One caller, `start` (0x40101F); in the game this never gets a call (the hook DLL is loaded
//! later). The Win32 call is made from Rust (the same in both programs).
//! `start` calls it directly (2026-10-23); the registers above are the original's, for
//! reference.

use crate::session::MACHINE;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardType;

// ---- game data ----

// ---- Rust version ----

/// The value the variable is given for a subtype, if any.
pub fn keyboard_type(subtype: u32) -> Option<u32> {
    // Converted: `rc2k_core::system::machine::keyboard_type`.
    rc2k_core::system::machine::keyboard_type(subtype)
}

pub unsafe fn read_keyboard_type() {
    let subtype = unsafe { GetKeyboardType(1) } as u32;
    if let Some(kind) = keyboard_type(subtype) {
        MACHINE.keyboard_type.set(kind);
    }
}
