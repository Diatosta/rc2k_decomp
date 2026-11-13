//! The fatal-error reporter.

pub mod raise;
pub mod report;
pub mod show_report_at_exit;

/// The reporter's variables.
pub mod state {
    use crate::session::{Byte, Bytes, Dword};

    /// g_fatalCode (0x4F52A4): the code of the error being reported.
    pub static CODE: Dword = Dword::new(0);
    /// g_fatalMessage (0x4F52A8 held a pointer to it, or 0): the error's message, or none.
    pub static MESSAGE: std::sync::Mutex<Option<&'static str>> = std::sync::Mutex::new(None);

    /// The message of the error being reported (`MESSAGE`).
    pub fn message() -> Option<&'static str> {
        *MESSAGE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// dword_4E0598 (0x4E0598): the Win32 error to name in a report, or 0 (nothing in the
    /// program sets it).
    pub static W32: Dword = Dword::new(0);
    /// g_fatalReported (0x4F67BC, a byte): 1 once a report was written.
    pub static REPORTED: Byte = Byte::new(0);
    /// `cursor` in IDA (0x4F67BD, 128 bytes): a string added to every report (empty).
    pub const EXTRA_LEN: usize = 128;
    pub static EXTRA: Bytes<EXTRA_LEN> = Bytes::new([0; EXTRA_LEN]);
    /// aEaxXxxxxxxxEbx (0x4F673A, 130 bytes): the registers' template, filled in place.
    pub const TEMPLATE_LEN: usize = 130;
    pub static TEMPLATE: Bytes<TEMPLATE_LEN> = Bytes::new(template());
    /// Text (0x9526F8, 0x400 bytes, bss): the report.
    pub const TEXT_LEN: usize = 0x400;
    pub static TEXT: Bytes<TEXT_LEN> = Bytes::new([0; TEXT_LEN]);
    /// Caption (0x4F66FD): "Rally: Guru Meditation", of which the log shows the part from
    /// +7 ("aGuruMeditation + 1"); byte 5 (byte_4F6702) is cut to 0 by the report's end.
    pub const CAPTION_LEN: usize = 23;
    pub static CAPTION: Bytes<CAPTION_LEN> = Bytes::new(*b"Rally: Guru Meditation\0");

    const fn template() -> [u8; TEMPLATE_LEN] {
        let text = b"EAX=XXXXXXXX\tEBX=XXXXXXXX\tECX=XXXXXXXX\tEDX=XXXXXXXX\r\nESI=XXXXXXXX\tEDI=XXXXXXXX\tESP=XXXXXXXX\tEBP=XXXXXXXX\r\nC=X  Z=X  S=X  V=X  P=X";
        let mut out = [0u8; TEMPLATE_LEN];
        let mut n = 0;
        while n < text.len() {
            out[n] = text[n];
            n += 1;
        }
        out
    }
}
