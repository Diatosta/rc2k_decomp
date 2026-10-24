//! `system::sleep` (0x41100C): `Sleep(eax)`.
//!
//! In:  eax = milliseconds
//! Out: every register preserved
//! 34 callers. Hook removed 2026-10-22 (direct calls): Rust callers run `sleep`
//! (by address: `routes.rs`); the registers above are the original's, for reference.

use windows_sys::Win32::System::Threading::Sleep;

// ---- Rust version ----

pub fn sleep(milliseconds: u32) {
    unsafe { Sleep(milliseconds) };
}
