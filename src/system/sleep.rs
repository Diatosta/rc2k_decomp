//! `system::sleep` (0x41100C): `Sleep(milliseconds)`.
//!
//! Takes milliseconds. 34 callers.

/// (`std::thread::sleep`, which is `Sleep` on Windows, 0 a yield as there.)
pub fn sleep(milliseconds: u32) {
    std::thread::sleep(std::time::Duration::from_millis(milliseconds.into()));
}
