//! `system::sleep` (0x41100C): `Sleep(eax)`.
//!
//! In:  eax = milliseconds
//! Out: every register preserved
//! 34 callers. Hook removed 2026-10-22 (direct calls): Rust callers run `sleep`
//! (by address: `routes.rs`); the registers above are the original's, for reference.

// ---- Rust version ----

/// (Phase 6: `std::thread::sleep`, which is `Sleep` on Windows, 0 a yield as there.)
pub fn sleep(milliseconds: u32) {
    std::thread::sleep(std::time::Duration::from_millis(milliseconds.into()));
}
