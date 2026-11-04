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

use crate::session::Dword;

// ---- game data ----

// ---- Rust version ----

/// SEM_FAILCRITICALERRORS.
const FAIL_CRITICAL_ERRORS: u32 = 1;

pub fn set_error_mode() {
    let was = crate::os::set_error_mode(FAIL_CRITICAL_ERRORS);
    ERROR_MODE.mode.set(was);
    ERROR_MODE.changed.set(1);
}

/// The error mode kept and whether it was set (IDA 0x4E059C .. 0x4E05A4, the image's layout).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ErrorMode {
    /// 0x4E059C: `g_errorModeSet`: 1 after `set_error_mode` set the mode.
    pub changed: Dword,
    /// 0x4E05A0: `uMode`: the mode before.
    pub mode: Dword,
}

pub static ERROR_MODE: ErrorMode = ErrorMode {
    changed: Dword::new(0x0),
    mode: Dword::new(0x0),
};

impl ErrorMode {
    pub const START: usize = 0x4E059C;
    pub const LEN: usize = 0x8;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const ERROR_MODE) as usize
    }
}

#[cfg(test)]
mod error_mode_tests {
    use super::*;

    #[test]
    fn errormode_starts_as_the_games() {
        let size = size_of::<ErrorMode>();
        assert!(ErrorMode::LEN <= size && size <= (ErrorMode::LEN).next_multiple_of(4));
        let image = crate::moved::image_bytes(ErrorMode::START, ErrorMode::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(ErrorMode::home()).cast_const(),
                ErrorMode::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
