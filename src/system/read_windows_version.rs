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

// ---- game data ----

const VERSION_INFO_LEN: usize = 0x94;

// ---- Rust version ----

pub fn read_windows_version(system: &crate::system::owner::System) {
    let record = crate::os::version_record();
    debug_assert_eq!(record.len(), VERSION_INFO_LEN);
    use crate::pointers::Store as _;
    system
        .session
        .machine
        .version
        .store(0, &record)
        .unwrap_or_else(|e| crate::fatal::raise::end("system::read_windows_version", e));
}
