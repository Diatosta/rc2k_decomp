//! `system::set_error_mode` (0x402B7C): `SetErrorMode(SEM_FAILCRITICALERRORS)` (no
//! "insert a disc" boxes from Windows), the mode before it kept in `uMode` (0x4E05A0), and
//! `dword_4E059C` 1.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! One caller, `start` (0x401035); in the game this never gets a call (the hook DLL is
//! loaded later). The Win32 call is made from Rust (the same in both programs). Manual
//! patch with a private copy (0x19 bytes, the call relocated): the call starts at +3.
//! Shadow mode runs the original alone (it changes the process's mode). Self-test: the
//! original and the Rust version from the same mode, both variables compared; the mode and
//! the variables put back.

use crate::framework::rva;
use windows_sys::Win32::System::Diagnostics::Debug::{SEM_FAILCRITICALERRORS, SetErrorMode};

// ---- game data ----

const KEPT_MODE_RVA: usize = rva(0x4E05A0); // uMode
const MODE_SET_RVA: usize = rva(0x4E059C); // dword_4E059C

// ---- Rust version ----

pub unsafe fn set_error_mode() {
    unsafe {
        let was = SetErrorMode(SEM_FAILCRITICALERRORS);
        crate::framework::game_write(KEPT_MODE_RVA, was);
        crate::framework::game_write(MODE_SET_RVA, 1u32);
    }
}
