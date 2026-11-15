//! `fatal::report` (0x40763C): the fatal-error and breakpoint reporter. About 190 functions
//! jump here, each from a small stub that has stored a code in `g_fatalCode` (0x4F52A4) and a message
//! pointer in `g_fatalMessage` (0x4F52A8).
//!
//! Only the first report of a run is written (`g_fatalReported`, 0x4F67BC). It builds this text in `Text`
//! (0x9526F8, 0x400 bytes):
//!   unless the code has bit 28: "Error code [XXXXXXXX]" (bit 31) or "Breakpoint [XXXXXXXX]";
//!   the message, if there is one, with the two characters `\n` turned into a line break;
//!   the string at 0x4F67BD;
//!   unless the code has bit 28: "Last file: `path'" if a file is open, "W32=XXXXXXXX" if
//!   the report's flag is not 0, and the registers and five flags, which are written into the
//!   template at 0x4F673A first.
//! Then `g_logEnabled` = 1 and "Guru Meditation" and the text go to the log.
//! After that, also when it was not the first report, a code with bit 31 asks the game to
//! quit (`threading::request_quit`) and jumps: to the exit path in `start` if the current
//! thread has no thread slot, else to that slot's start routine, with the slot as its argument.
//!
//! It is called directly (`report_now`, from `fatal::raise`), and its
//! two ways out are Rust's: the program ended (`standalone::finish`), or the slot's
//! routine run again and the thread ended. A text that does not fit
//! `Text` ends the program, saying so.

use crate::errors::BadInput;
use crate::fatal::state::{CAPTION, CODE, EXTRA, REPORTED, TEMPLATE, TEXT, W32};
use crate::logfile::program_log::log;
use crate::threading::find_thread_slot::find_thread_slot;
use crate::threading::request_quit::request_quit_in;
use std::sync::atomic::Ordering;

// ---- game data ----

// The variables: `crate::fatal::state`. "Guru Meditation" is the caption from +7.
// threading::request_quit: asks the game to quit, once

pub use rc2k_core::fatal::{Context, Leave, TEXT_ONLY, register_dump, report_text};

/// The report for what the game's variables hold now.
fn text_from_game(context: Option<&Context>) -> Vec<u8> {
    let message = crate::fatal::state::message().map(str::as_bytes);
    let last_file = crate::files::state::last_file_path();
    report_text(
        CODE.get(),
        message,
        &EXTRA.text(),
        last_file.as_deref(),
        W32.get(),
        context,
    )
}

/// What the reporter needs from code that is not ported, or from a test.
pub struct Outside<'a> {
    /// Logs a NUL-terminated text at level 1 with a line break.
    pub log: &'a mut dyn FnMut(&[u8]),
    /// Asks the game to quit.
    pub quit: &'a mut dyn FnMut(),
    /// The current thread's slot (one more than its number), or 0.
    pub thread_slot: &'a dyn Fn() -> u32,
}

pub fn report(context: Option<&Context>, outside: &mut Outside) -> Result<Leave, BadInput> {
    let code: u32 = CODE.get();
    let text = text_from_game(context);
    // Converted: `rc2k_core::fatal` (`report_text`, `fits`, `leave`).
    if !rc2k_core::fatal::fits(text.len()) {
        return Err(BadInput("the report does not fit its buffer"));
    }
    if REPORTED.atomic().swap(1, Ordering::SeqCst) != 1 {
        if let (true, Some(context)) = (code & TEXT_ONLY == 0, context) {
            // The original fills in its template in place, so it keeps the values.
            let dump = register_dump(context);
            TEMPLATE.write_at(0, &dump, "the registers' template")?;
        }
        TEXT.write_at(0, &text, "the report")?;
        TEXT.write_at(text.len(), &[0], "the report")?;
        crate::logfile::write_line::set_enabled(1); // g_logEnabled
        (outside.log)(&CAPTION.read()[7..]);
        (outside.log)(&text);
    }
    Ok(rc2k_core::fatal::leave(
        code,
        outside.quit,
        outside.thread_slot,
        // The slot (one more than its number) holds its routine as an IDA address.
        &|slot| {
            crate::threading::thread_slot::slot_ref(slot as usize - 1)
                .unwrap_or_else(|e| crate::fatal::raise::end("fatal::report: a thread slot", e))
                .routine
                .get()
        },
    ))
}

/// The reporter called directly by `fatal::raise`, with the registers of the place that
/// refused when they are known (the report has no register lines otherwise). A code
/// without the fatal bit returns; with it the game is asked to quit and this thread ends
/// as the original's two ways out end it: with no thread slot the program at `start`'s way
/// out, on a slot's thread its routine run again, which finds the game
/// quitting, and the thread ended.
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
        thread_slot: &find_thread_slot,
    };
    let code = CODE.get();
    match report(context, &mut outside) {
        Ok(Leave::Return) => {}
        Ok(Leave::Exit) => crate::standalone::finish(
            &format!("fatal error {code:#x}: the game's way out (`start`, 0x4010EE)"),
            0,
        ),
        Ok(Leave::Routine { slot, routine }) => {
            log(&format!(
                "fatal::report: fatal error {code:#x} on the thread of slot {slot:#x}: its routine {routine:#x} again"
            ));
            let ran = owners
                .and_then(|owners| crate::threading::thread_slot::run_routine(routine, owners));
            match ran {
                Some(_) => crate::platform::sync::exit_thread(),
                None => crate::standalone::finish(
                    &format!("not ported: the thread routine {routine:#x} (fatal error {code:#x})"),
                    1,
                ),
            }
        }
        Err(why) => crate::standalone::finish(
            &format!("fatal::report refused ({}) on fatal error {code:#x}", why.0),
            1,
        ),
    }
}
