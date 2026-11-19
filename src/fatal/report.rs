//! The fatal-error and breakpoint reporter.
//!
//! Only the first report of a run is written. Its text (`rc2k_core::fatal::report_text`):
//!   unless the code has bit 28: "Error code [XXXXXXXX]" (bit 31) or "Breakpoint [XXXXXXXX]";
//!   the message, if there is one, with the two characters `\n` turned into a line break;
//!   unless the code has bit 28: "Last file: `path'" if a file is open, and the registers and
//!   five flags where the place that failed gave them.
//! Then the game's log is switched on and "Guru Meditation" and the text go to it.
//! After that, also when it was not the first report, a code with bit 31 asks the game to
//! quit (`threading::request_quit`) and leaves: the program ended (`standalone::finish`) on a
//! thread that is none of the game's own four, else that thread's routine run again (it finds
//! the game quitting and cleans up) and the thread ended. A text that does not fit `TEXT` ends
//! the program, saying so.

use crate::errors::BadInput;
use crate::fatal::state::{CAPTION, CODE, REPORTED, TEXT};
use crate::program_log::log;
use crate::threading::request_quit::request_quit_in;
use crate::threading::thread_slot::SlotThread;
use std::sync::atomic::Ordering;

pub use rc2k_core::fatal::{Context, Leave, report_text};

/// Where "Guru Meditation" starts in the caption.
const CAPTION_NAME_AT: usize = 7;

/// The report for what the reporter's variables hold now.
fn text_from_game(context: Option<&Context>) -> Vec<u8> {
    let message = crate::fatal::state::message().map(str::as_bytes);
    let last_file = crate::files::state::last_file_path();
    report_text(CODE.get(), message, last_file.as_deref(), context)
}

/// What the reporter needs from outside, so that a test can stand in for it.
pub struct Outside<'a> {
    /// Logs a NUL-terminated text at level 1 with a line break.
    pub log: &'a mut dyn FnMut(&[u8]),
    /// Asks the game to quit.
    pub quit: &'a mut dyn FnMut(),
    /// Which of the game's own threads the current one is, if any.
    pub own_thread: &'a dyn Fn() -> Option<SlotThread>,
}

pub fn report(
    context: Option<&Context>,
    outside: &mut Outside,
) -> Result<Leave<SlotThread>, BadInput> {
    let code: u32 = CODE.get();
    let text = text_from_game(context);
    if !rc2k_core::fatal::fits(text.len()) {
        return Err(BadInput("the report does not fit its buffer"));
    }
    if REPORTED.atomic().swap(1, Ordering::SeqCst) != 1 {
        TEXT.write_at(0, &text, "the report")?;
        TEXT.write_at(text.len(), &[0], "the report")?;
        crate::logfile::write_line::set_enabled(1);
        (outside.log)(&CAPTION.read()[CAPTION_NAME_AT..]);
        (outside.log)(&text);
    }
    Ok(rc2k_core::fatal::leave(
        code,
        outside.quit,
        outside.own_thread,
    ))
}

/// The reporter called by `fatal::raise`, with the registers of the place that refused when
/// they are known (the report has no register lines otherwise). A code without the fatal bit
/// returns; with it the game is asked to quit and this thread ends (see the header).
pub fn report_now(context: Option<&Context>) {
    log(&format!(
        "fatal::report: thread {}",
        crate::platform::sync::thread_id()
    ));
    // The reporter runs on any thread with no owners in hand, even before the start has
    // published them: it reads the one published handle, the only global of owners that stays.
    let owners = crate::threading::thread_slot::published();
    let mut outside = Outside {
        log: &mut |text| crate::logfile::log_write::log_nul(text, true, 1),
        // If no shutdown thread can be created there is nothing more to report: a report
        // is being made already, and the way out below ends the game anyway.
        quit: &mut || {
            if let Some(owners) = owners {
                let _ = request_quit_in(owners);
            }
        },
        own_thread: &SlotThread::current,
    };
    let code = CODE.get();
    // The player is told before the report asks the game to quit (a quitting game shows
    // nothing).
    if code & rc2k_core::fatal::FATAL != 0 && REPORTED.get() != 1 {
        crate::error_dialog::fatal(&rc2k_core::files::ansi::ansi_text(&text_from_game(context)));
    }
    match report(context, &mut outside) {
        Ok(Leave::Return) => {}
        Ok(Leave::Exit) => crate::standalone::finish(&format!("fatal error {code:#x}"), 0),
        Ok(Leave::Routine(thread)) => {
            let name = thread.name();
            log(&format!(
                "fatal::report: fatal error {code:#x} on the {name} thread: its routine again"
            ));
            let Some(owners) = owners else {
                crate::standalone::finish(
                    &format!("fatal error {code:#x} on the {name} thread before the start"),
                    1,
                )
            };
            // What the routine gives is not looked at: a report is being made already.
            let _ = thread.run(owners);
            crate::platform::sync::exit_thread()
        }
        Err(why) => crate::standalone::finish(
            &format!("fatal::report refused ({}) on fatal error {code:#x}", why.0),
            1,
        ),
    }
}
