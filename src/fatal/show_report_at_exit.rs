//! At the program's end, if a fatal error was reported, its text (written by `fatal::report`)
//! shown in a message box under the caption "Rally: Guru Meditation"; for a code with bit 28
//! the caption is first cut to "Rally". Then "END PROCESS" logged.

use crate::fatal::state::{CAPTION, CODE, REPORTED, TEXT};
use crate::logfile::write_line::log_text;
pub fn show_report_at_exit() {
    if REPORTED.get() == 1 {
        if rc2k_core::fatal::caption_cut(CODE.get()) {
            CAPTION.set(rc2k_core::fatal::CAPTION_CUT, 0);
        }
        let (caption, text) = (CAPTION.read(), TEXT.read());
        let until_nul = |b: &[u8]| b.split(|&c| c == 0).next().unwrap_or_default().to_vec();
        crate::platform::os::message_box(
            crate::platform::os::Icon::Warning,
            &until_nul(&caption),
            &until_nul(&text),
        );
    }
    log_text(b"END PROCESS", true, 2);
}
