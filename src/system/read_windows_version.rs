//! `system::read_windows_version` (0x410AC5): `VersionInformation` (0x518408, an
//! `OSVERSIONINFOA` of 0x94 bytes) filled by `GetVersionExA`, its size set first.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! What `logfile::start_log` writes to the log ("Windows" major.minor.build). One caller,
//! `start` (0x401024); in the game this never gets a call (the hook DLL is loaded later). The
//! Win32 call is made from Rust (the same in both programs).
//! `start` calls it directly (2026-10-23); the registers above are the original's, for
//! reference.

use crate::framework::{game_addr, rva};
use windows_sys::Win32::System::SystemInformation::{GetVersionExA, OSVERSIONINFOA};

// ---- game data ----

const VERSION_INFO_RVA: usize = rva(0x518408); // VersionInformation
const VERSION_INFO_LEN: usize = 0x94;

// ---- Rust version ----

pub unsafe fn read_windows_version() {
    let info = game_addr(VERSION_INFO_RVA) as *mut OSVERSIONINFOA;
    unsafe {
        (info as *mut u32).write_unaligned(VERSION_INFO_LEN as u32);
        GetVersionExA(info);
    }
}
