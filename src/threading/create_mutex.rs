//! `threading::create_mutex` (0x41107F): a mutex without a name and without an owner,
//! `CreateMutexA(0, 0, 0)`.
//!
//! Returns the handle, or none (the original's carry flag).
//! One caller, `net::set_up` (0x4184BF), which raises fatal error 0x80168502 on the carry.
//! The Win32 call is made from Rust.

// ---- Rust version ----

/// The handle, 0 for none.
pub fn create_mutex() -> u32 {
    crate::sync::create_mutex()
}
