//! `SetErrorMode(SEM_FAILCRITICALERRORS)` (no "insert a disc" boxes from Windows), the mode
//! before it kept in `mode`, and `changed` 1. Called by `start`.

use crate::memory::session::PlainDword;
use crate::system::owner::System;

/// SEM_FAILCRITICALERRORS.
const FAIL_CRITICAL_ERRORS: u32 = 1;

pub fn set_error_mode(system: &System) {
    let was = crate::platform::os::set_error_mode(FAIL_CRITICAL_ERRORS);
    system.start().error_mode.mode.set(was);
    system.start().error_mode.changed.set(1);
}

/// The error mode kept and whether it was set.
#[derive(rc2k_derive::Cells)]
pub struct ErrorMode {
    /// 1 after `set_error_mode` set the mode.
    pub changed: PlainDword,
    /// The mode before.
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
