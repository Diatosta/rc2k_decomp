//! `threading::create_event` (0x41103C): an event without a name,
//! `CreateEventA(0, eax, ebx, 0)`.
//!
//! In:  eax = manual reset (0 or 1), ebx = signalled at first (0 or 1)
//! Out: eax = the handle; `cmp eax, 1`: carry set (and zero clear) if there is none; ecx,
//!      edx pushed and popped, the rest kept by Win32
//! One caller, `net::set_up` (0x4184AF: eax 0, ebx 0), which raises fatal error 0x80168501
//! on the carry. The Win32 call is made from Rust (the same in both programs). Shadow mode
//! runs the original alone (an event cannot be made twice). Self-test: both make events of
//! each kind, the handles' state read (`WaitForSingleObject` with 0 ms), then closed.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use windows_sys::Win32::System::Threading::CreateEventA;

// ---- Rust version ----

/// The handle, 0 for none.
pub fn create_event(manual_reset: u32, signalled: u32) -> u32 {
    unsafe {
        CreateEventA(
            std::ptr::null(),
            manual_reset as i32,
            signalled as i32,
            std::ptr::null(),
        ) as u32
    }
}

/// `cmp eax, 1` on a handle: (carry, zero).
// Converted: `rc2k_core::threading::handle_flags`.
pub use rc2k_core::threading::handle_flags;
