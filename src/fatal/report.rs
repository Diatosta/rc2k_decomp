//! `fatal::report` (0x40763C): the fatal-error and breakpoint reporter. About 190 functions
//! jump here, each from a small stub that has stored a code in `g_fatalCode` (0x4F52A4) and a message
//! pointer in `g_fatalMessage` (0x4F52A8).
//!
//! In:  entered by `jmp`, with the registers and flags of the place that failed
//! Out: for a code without bit 31, returns (to whoever called the function that jumped here),
//!      with every register as it was; for a code with bit 31, does not return
//! Only the first report of a run is written (`g_fatalReported`, 0x4F67BC). It builds this text in `Text`
//! (0x9526F8, 0x400 bytes):
//!   unless the code has bit 28: "Error code [XXXXXXXX]" (bit 31) or "Breakpoint [XXXXXXXX]";
//!   the message, if there is one, with the two characters `\n` turned into a line break;
//!   the string at 0x4F67BD;
//!   unless the code has bit 28: "Last file: `path'" if a file is open, "W32=XXXXXXXX" if
//!   `dword_4E0598` is not 0, and the registers and five flags, which are written into the
//!   template at 0x4F673A first.
//! Then `g_logEnabled` = 1 and "Guru Meditation" and the text go to the log.
//! After that, also when it was not the first report, a code with bit 31 asks the game to
//! quit (`threading::request_quit`, 0x410BF9) and jumps: to the exit path in `start` (0x4010EE) if the current
//! thread has no thread slot (esi is then 0), else to that slot's start routine, with esi =
//! the slot.
//!
//! Since 2026-10-24 it is called directly (`report_now`, from `fatal::raise`), and its
//! two ways out are Rust's: the program ended (`standalone::finish`), or the slot's
//! routine (its Rust version) run again and the thread ended. A text that does not fit
//! `Text` ends the program, saying so.

use crate::errors::BadInput;
use crate::fatal::state::{CAPTION, CODE, EXTRA, MESSAGE, REPORTED, TEMPLATE, TEXT, W32};
use crate::framework::{RawHook, Regs, log, rva};
use crate::menu::text_entry::log_l1;
use crate::threading::find_thread_slot::find_thread_slot;
use crate::threading::request_quit::request_quit;
use std::ffi::{CStr, c_char};
use std::sync::atomic::Ordering;

pub static HOOK: RawHook = RawHook::new("fatal::report", rva(0x40763C));

const _: () = assert!(size_of::<Regs>() == 36);

// ---- game data ----

// The variables: `crate::fatal::state`. "Guru Meditation" is the caption from +7.
// threading::request_quit: asks the game to quit, once

// ---- Rust version ----

pub use rc2k_core::fatal::{Context, Leave, TEXT_ONLY, register_dump, report_text};

unsafe fn cstr<'a>(ptr: *const u8) -> &'a [u8] {
    unsafe { CStr::from_ptr(ptr as *const c_char) }.to_bytes()
}

/// The report for what the game's variables hold now.
unsafe fn text_from_game(context: &Context) -> Vec<u8> {
    unsafe {
        let message: u32 = MESSAGE.load(Ordering::Relaxed);
        let last_file: u32 =
            crate::files::state::LAST_FILE_PATH.load(std::sync::atomic::Ordering::Relaxed);
        report_text(
            CODE.load(Ordering::Relaxed),
            (message != 0).then(|| cstr(crate::pointers::host(message) as *const u8)),
            cstr(EXTRA.ptr().cast()),
            (last_file != 0).then(|| cstr(crate::pointers::host(last_file) as *const u8)),
            W32.load(Ordering::Relaxed),
            context,
        )
    }
}

/// What the reporter needs from code that is not ported, or from a test.
pub struct Outside<'a> {
    /// Logs a NUL-terminated text at level 1 with a line break.
    pub log: &'a mut dyn FnMut(*const u8),
    /// Asks the game to quit.
    pub quit: &'a mut dyn FnMut(),
    /// The current thread's slot, or 0.
    pub thread_slot: &'a dyn Fn() -> u32,
}

pub unsafe fn report(context: &Context, outside: &mut Outside) -> Result<Leave, BadInput> {
    unsafe {
        let code: u32 = CODE.load(Ordering::Relaxed);
        let text = text_from_game(context);
        // Converted: `rc2k_core::fatal` (`report_text`, `fits`, `leave`).
        if !rc2k_core::fatal::fits(text.len()) {
            return Err(BadInput("the report does not fit its buffer"));
        }
        if REPORTED.swap(1, Ordering::SeqCst) != 1 {
            if code & TEXT_ONLY == 0 {
                // The original fills in its template in place, so it keeps the values.
                let dump = register_dump(context);
                std::ptr::copy_nonoverlapping(dump.as_ptr(), TEMPLATE.ptr().cast(), dump.len());
            }
            let buffer: *mut u8 = TEXT.ptr().cast();
            std::ptr::copy_nonoverlapping(text.as_ptr(), buffer, text.len());
            buffer.add(text.len()).write(0);
            crate::logfile::write_line::set_enabled(1); // g_logEnabled
            (outside.log)(CAPTION.ptr().cast::<u8>().add(7));
            (outside.log)(buffer);
        }
        Ok(rc2k_core::fatal::leave(
            code,
            outside.quit,
            outside.thread_slot,
            // The slot holds its routine as an IDA address.
            &|slot| (crate::pointers::host(slot) as *const u32).read_unaligned(),
        ))
    }
}

/// The reporter called directly by `fatal::raise` with the registers of the place that
/// refused (2026-10-24: no longer by its address through the fault handler). A code
/// without the fatal bit returns; with it the game is asked to quit and this thread ends
/// as the original's two ways out end it: with no thread slot the program at `start`'s way
/// out, on a slot's thread its routine (the Rust version) run again, which finds the game
/// quitting, and the thread ended.
pub fn report_now(r: &Regs) {
    HOOK.count_call();
    let context = Context {
        eax: r.eax,
        ebx: r.ebx,
        ecx: r.ecx,
        edx: r.edx,
        esi: r.esi,
        edi: r.edi,
        esp: r.esp,
        ebp: r.ebp,
        flags: r.eflags,
    };
    log(&format!(
        "fatal::report: thread {}",
        crate::sync::thread_id()
    ));
    let mut outside = Outside {
        log: &mut |text| unsafe { log_l1(text, true) },
        // If no shutdown thread can be created there is nothing more to report: a report
        // is being made already, and the way out below ends the game anyway.
        quit: &mut || {
            let _ = request_quit();
        },
        thread_slot: &|| unsafe { crate::pointers::game_address(find_thread_slot()) },
    };
    let code = CODE.load(Ordering::Relaxed);
    match unsafe { report(&context, &mut outside) } {
        Ok(Leave::Return) => {}
        Ok(Leave::Exit) => crate::standalone::finish(
            &format!("fatal error {code:#x}: the game's way out (`start`, 0x4010EE)"),
            0,
        ),
        Ok(Leave::Routine { slot, routine }) => {
            log(&format!(
                "fatal::report: fatal error {code:#x} on the thread of slot {slot:#x}: its routine {routine:#x} again"
            ));
            match crate::threading::thread_slot::run_routine(routine) {
                Some(_) => crate::sync::exit_thread(),
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

// ---- self-test: a copy of the original that cannot end the game ----

// pusha; pushf; pop edx; mov ebp, esp
// what the stand-in for find_thread_slot returns
// a slot: its first dword is the routine
