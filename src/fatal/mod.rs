//! The fatal-error reporter.

pub(crate) mod raise;
mod report;
pub(crate) mod show_report_at_exit;

/// The reporter's variables.
pub mod state {
    use crate::memory::session::{Byte, Bytes, Dword};

    /// The code of the error being reported.
    pub static CODE: Dword = Dword::new(0);
    /// The error's message, or none.
    pub static MESSAGE: std::sync::Mutex<Option<&'static str>> = std::sync::Mutex::new(None);

    /// The message of the error being reported (`MESSAGE`).
    pub fn message() -> Option<&'static str> {
        *MESSAGE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// 1 once a report was written.
    pub static REPORTED: Byte = Byte::new(0);
    /// The report, NUL-terminated.
    pub const TEXT_LEN: usize = 0x400;
    pub static TEXT: Bytes<TEXT_LEN> = Bytes::new([0; TEXT_LEN]);
    /// The message box's caption; the log shows the part after "Rally: ", and a report of the
    /// message alone cuts it to "Rally".
    pub const CAPTION_LEN: usize = 23;
    pub static CAPTION: Bytes<CAPTION_LEN> = Bytes::new(*b"Rally: Guru Meditation\0");
}
