//! The Windows version record (`OSVERSIONINFOA`, 0x94 bytes) filled, its size set first. What
//! `logfile::start_log` writes to the log ("Windows" major.minor.build).

const VERSION_INFO_LEN: usize = 0x94;

pub fn read_windows_version(system: &crate::system::owner::System) {
    let record = crate::platform::os::version_record();
    debug_assert_eq!(record.len(), VERSION_INFO_LEN);
    for (dword, bytes) in system
        .machine()
        .version
        .iter()
        .zip(record.as_chunks::<4>().0)
    {
        dword.set(u32::from_le_bytes(*bytes));
    }
}
