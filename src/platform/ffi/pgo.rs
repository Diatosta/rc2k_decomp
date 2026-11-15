//! The profile runtime's call (linked in by `-C profile-generate`, the PGO build only).

#![allow(unsafe_code)]

unsafe extern "C" {
    fn __llvm_profile_write_file() -> i32;
}

/// Writes the profiler runtime's counters file: it touches nothing of ours. `terminate`
/// skips the `atexit` that would call it, so `standalone::finish` calls this first.
pub fn write_profile() {
    // SAFETY: the profiler runtime's own call; it writes its counters' file and touches
    // nothing of ours.
    unsafe {
        __llvm_profile_write_file();
    }
}
