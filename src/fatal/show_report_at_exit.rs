//! `fatal::show_report_at_exit` (0x407855): at the game's end, if a fatal error was
//! reported (`g_fatalReported` 1), its text (`Text`, written by `fatal::report`) shown in
//! a message box (`MessageBoxA`, no window, caption `Caption`, type 0x2030: an
//! exclamation, task-modal); for a code with bit 28 the caption ("Rally: Guru Meditation")
//! is first cut to "Rally" (its byte 5 made 0). Then "END PROCESS" logged.
//!
//! One caller, `start`'s way out, which calls it directly. The registers
//! above are the original's, for reference.

use crate::fatal::state::{CAPTION, CODE, REPORTED, TEXT};
use crate::logfile::write_line::log_text;
pub fn show_report_at_exit() {
    if REPORTED.get() == 1 {
        // Converted: `rc2k_core::fatal::caption_cut`.
        if rc2k_core::fatal::caption_cut(CODE.get()) {
            CAPTION.set(rc2k_core::fatal::CAPTION_CUT, 0);
        }
        // MB_ICONEXCLAMATION | MB_TASKMODAL.
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
