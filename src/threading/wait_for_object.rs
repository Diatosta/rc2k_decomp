//! `threading::wait_for_object` (0x411063): `WaitForSingleObject(handle, milliseconds)`.
//!
//! Takes: the handle, milliseconds
//! One caller, `net::receive_thread` (0x4183F4: the network's event, 1000 ms), which reads
//! only the zero flag (a failure counts as the time running out). The Win32 call is made
//! from Rust.

// ---- Rust version ----

pub use rc2k_core::threading::Waited;

pub fn wait_for_object(handle: u32, milliseconds: u32) -> Waited {
    // Converted: `rc2k_core::threading::Waited`.
    Waited::of(crate::sync::wait(handle, milliseconds))
}
