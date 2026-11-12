//! `system::set_error_mode` (0x402B7C): `SetErrorMode(SEM_FAILCRITICALERRORS)` (no
//! "insert a disc" boxes from Windows), the mode before it kept in `uMode` (0x4E05A0), and
//! `changed` 1.
//!
//! One caller, `start` (0x401035). The Win32 call is made from Rust.

use crate::session::PlainDword;
use crate::system::owner::System;

// ---- game data ----

/// SEM_FAILCRITICALERRORS.
const FAIL_CRITICAL_ERRORS: u32 = 1;

pub fn set_error_mode(system: &System) {
    let was = crate::os::set_error_mode(FAIL_CRITICAL_ERRORS);
    system.start().error_mode.mode.set(was);
    system.start().error_mode.changed.set(1);
}

/// The error mode kept and whether it was set (IDA 0x4E059C .. 0x4E05A4, the image's layout).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ErrorMode {
    /// 0x4E059C: `g_errorModeSet`: 1 after `set_error_mode` set the mode.
    pub changed: PlainDword,
    /// 0x4E05A0: `uMode`: the mode before.
    pub mode: PlainDword,
}

impl ErrorMode {
    pub const fn new() -> Self {
        ErrorMode {
            changed: PlainDword::new(0x0),
            mode: PlainDword::new(0x0),
        }
    }
}
