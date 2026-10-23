//! `fatal::show_report_at_exit` (0x407855): at the game's end, if a fatal error was
//! reported (`g_fatalReported` 1), its text (`Text`, written by `fatal::report`) shown in
//! a message box (`MessageBoxA`, no window, caption `Caption`, type 0x2030: an
//! exclamation, task-modal); for a code with bit 28 the caption ("Rally: Guru Meditation")
//! is first cut to "Rally" (its byte 5, `byte_4F6702`, 0). Then "END PROCESS" logged.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out, which calls it directly (2026-10-23). The registers
//! above are the original's, for reference.

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::logfile::write_line::log_text;

const REPORTED_RVA: usize = rva(0x4F67BC); // g_fatalReported (a byte)
const CODE_RVA: usize = rva(0x4F52A4); // g_fatalCode
const CAPTION_RVA: usize = rva(0x4F66FD); // Caption; byte_4F6702 its sixth byte
const TEXT_RVA: usize = rva(0x9526F8); // Text: the report

/// The Rust version (see the header).
pub fn show_report_at_exit() {
    use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA;
    unsafe {
        if game_read::<u8>(REPORTED_RVA) == 1 {
            // Converted: `rc2k_core::fatal::caption_cut`.
            if rc2k_core::fatal::caption_cut(game_read(CODE_RVA)) {
                game_write(CAPTION_RVA + rc2k_core::fatal::CAPTION_CUT, 0u8);
            }
            MessageBoxA(
                std::ptr::null_mut(),
                game_addr(TEXT_RVA) as _,
                game_addr(CAPTION_RVA) as _,
                0x2030,
            );
        }
    }
    log_text(b"END PROCESS", true, 2);
}
