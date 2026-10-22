//! `threading::wait_for_object` (0x411063): `WaitForSingleObject(eax, ebx)`.
//!
//! In:  eax = the handle, ebx = milliseconds
//! Out: every register preserved (eax, ecx, edx pushed and popped). Failed (`WAIT_FAILED`):
//!      carry and zero set. Else carry clear and zero set if the time ran out
//!      (`WAIT_TIMEOUT`, 0x102), clear if the object was signalled.
//! One caller, `net::receive_thread` (0x4183F4: the network's event, 1000 ms), which reads
//! only the zero flag (a failure counts as the time running out). The Win32 call is made
//! from Rust (the same in both programs). The thunk is not gated (the call waits). Shadow
//! mode runs the original alone (a wait takes what it waits for). Self-test: events of the
//! test's, signalled and not, and no handle, 0 ms each.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use windows_sys::Win32::System::Threading::WaitForSingleObject;

// ---- Rust version ----

pub use rc2k_core::threading::Waited;

pub fn wait_for_object(handle: u32, milliseconds: u32) -> Waited {
    // Converted: `rc2k_core::threading::Waited`.
    Waited::of(unsafe { WaitForSingleObject(handle as _, milliseconds) })
}
