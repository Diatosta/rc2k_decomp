//! `memory::virtual_alloc` (0x402B1A): eax bytes of committed read-write memory from
//! `VirtualAlloc`, or none for 0 bytes.
//!
//! In:  eax = bytes
//! Out: eax = the memory (0 for 0 bytes); carry clear; ecx, edx pushed and popped, the rest
//!      kept by Win32. None to be had: fatal error 0x80108571 "Need more virtual memory!"
//! Two callers: `memory::allocate` (whose Rust version makes the call itself) and
//! `memory::open_own_heap`. Manual patch with a private copy (0x21 bytes; the call and the
//! `jb` to the fatal stub relocated): the `jz` at +4 is in the first bytes. Shadow mode runs
//! the original alone (memory cannot be taken twice). Self-test: 0 bytes, and some sizes
//! from each, the memory checked (page-aligned, zeroed, writable) and given back.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{FatalError, Refused};
use windows_sys::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, VirtualAlloc};

pub const NO_MEMORY: FatalError = FatalError {
    code: 0x8010_8571,
    message: "Need more virtual memory!",
};

// ---- Rust version ----

/// The memory, 0 for 0 bytes.
pub unsafe fn virtual_alloc(bytes: u32) -> Result<u32, Refused> {
    if bytes == 0 {
        return Ok(0);
    }
    let at = unsafe {
        VirtualAlloc(
            std::ptr::null(),
            bytes as usize,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    } as u32;
    if at == 0 {
        return Err(NO_MEMORY.into());
    }
    Ok(at)
}
