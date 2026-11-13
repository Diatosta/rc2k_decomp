//! The Windows version record (`OSVERSIONINFOA`, 0x94 bytes) filled, its size set first. What
//! `logfile::start_log` writes to the log ("Windows" major.minor.build).

const VERSION_INFO_LEN: usize = 0x94;

pub fn read_windows_version(system: &crate::system::owner::System) {
    let record = crate::os::version_record();
    debug_assert_eq!(record.len(), VERSION_INFO_LEN);
    use crate::cells::Store as _;
    system
        .machine()
        .version
        .store(0, &record)
        .unwrap_or_else(|e| crate::fatal::raise::end("system::read_windows_version", e));
}
