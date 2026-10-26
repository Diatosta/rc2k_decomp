//! `threading::create_mutex` (0x41107F): a mutex without a name and without an owner,
//! `CreateMutexA(0, 0, 0)`.
//!
//! In:  nothing
//! Out: eax = the handle; `cmp eax, 1`: carry set (and zero clear) if there is none; ecx,
//!      edx pushed and popped, the rest kept by Win32
//! One caller, `net::set_up` (0x4184BF), which raises fatal error 0x80168502 on the carry.
//! The Win32 call is made from Rust (the same in both programs). Shadow mode runs the
//! original alone (a mutex cannot be made twice). Self-test: both make one, which another
//! thread can take at once (no owner), then closed.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use windows_sys::Win32::System::Threading::CreateMutexA;

// ---- Rust version ----

/// The handle, 0 for none.
pub fn create_mutex() -> u32 {
    unsafe { crate::pointers::game_address(CreateMutexA(std::ptr::null(), 0, std::ptr::null())) }
}
