//! `system::sleep` (0x41100C): `Sleep(eax)`.
//!
//! In:  eax = milliseconds
//! Out: every register preserved
//! 34 callers. Hook removed 2026-10-22 (direct calls): Rust callers run its `entry`
//! (`calls::call`, `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use windows_sys::Win32::System::Threading::Sleep;

// ---- Rust version ----

pub fn sleep(milliseconds: u32) {
    unsafe { Sleep(milliseconds) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    sleep(r.eax);
    Ok(())
}
