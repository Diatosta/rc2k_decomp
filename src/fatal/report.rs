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
//! The thunk is written by hand: the function is entered by a jump and may leave by one. It
//! reserves a stack slot that `dispatch` fills with where to go, and ends in a `ret` through
//! that slot: a stub that returns, the game's exit path, a thread routine, or the original.
//! Everything it calls is ported; only its two ways out of a fatal error are jumps into
//! original code.
//! Refuses (the original then runs): a text that does not fit `Text`.
//! Shadow mode cannot run both: the original runs, and what the Rust version would have
//! written is put in hook.log beforehand, to compare with the game's log by eye. The
//! self-test runs a copy of the original whose log, quit and thread-slot calls go to
//! stand-ins and whose two ways out land in the test.

use crate::errors::BadInput;
use crate::fatal::state::{CAPTION, CODE, EXTRA, MESSAGE, REPORTED, TEMPLATE, TEXT, W32};
use crate::framework::{RawHook, Regs, game_addr, log, rva};
use crate::menu::text_entry::log_l1;
use crate::threading::find_thread_slot::find_thread_slot;
use crate::threading::request_quit::request_quit;
use std::ffi::{CStr, c_char};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("fatal::report", rva(0x40763C), thunk, &ORIGINAL);

const _: () = assert!(size_of::<Regs>() == 36);

/// Detour target. The stack slot pushed first is filled by `dispatch` with the address the
/// final `ret` goes to; at that point every register and flag is as it was on entry (or as
/// `dispatch` set it) and the stack is as the game left it.
#[unsafe(naked)]
unsafe extern "C" fn thunk() {
    core::arch::naked_asm!(
        "push 0",              // where to go: dispatch fills it in
        "pushfd",              // Regs.eflags
        "pushad",
        "cld",
        "mov eax, esp",
        "push eax",            // arg: *mut Regs
        "call {dispatch}",
        "add esp, 4",
        "popad",
        "popfd",
        "ret",
        dispatch = sym dispatch,
    )
}

/// Where a non-fatal report goes: it leaves the flags of the original's last `test` (the
/// fatal bit, which is clear) and returns on the game's stack.
#[unsafe(naked)]
unsafe extern "C" fn return_to_game() {
    core::arch::naked_asm!("test dword ptr [{zero}], 0x80000000", "ret", zero = sym ZERO)
}

static ZERO: AtomicU32 = AtomicU32::new(0);

// ---- game data ----

// The variables: `crate::fatal::state`. "Guru Meditation" is the caption from +7.
// threading::request_quit: asks the game to quit, once
const EXIT_PATH_RVA: usize = rva(0x4010EE); // loc_4010EE in `start`

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
            (message != 0).then(|| cstr(message as *const u8)),
            cstr(EXTRA.ptr().cast()),
            (last_file != 0).then(|| cstr(last_file as *const u8)),
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
            &|slot| game_addr(rva((slot as *const u32).read_unaligned() as usize)) as u32,
        ))
    }
}

// ---- hook ----

/// The registers and flags the thunk saved, as the place that failed had them. The thunk
/// pushed two dwords before `pushad`, so the stack pointer was 8 higher.
fn context_of(r: &Regs) -> Context {
    Context {
        eax: r.eax,
        ebx: r.ebx,
        ecx: r.ecx,
        edx: r.edx,
        esi: r.esi,
        edi: r.edi,
        esp: r.esp.wrapping_add(8),
        ebp: r.ebp,
        flags: r.eflags,
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let context = context_of(r);
    let original = ORIGINAL.load(Ordering::SeqCst) as u32;
    log(&format!("fatal::report: thread {}", unsafe {
        windows_sys::Win32::System::Threading::GetCurrentThreadId()
    }));

    let target = {
        let mut outside = Outside {
            log: &mut |text| unsafe { log_l1(text, true) },
            // If no shutdown thread can be created there is nothing more to report: a report
            // is being made already, and the way out below ends the game anyway.
            quit: &mut || unsafe {
                let _ = request_quit();
            },
            thread_slot: &|| unsafe { find_thread_slot() as u32 },
        };
        match unsafe { report(&context, &mut outside) } {
            Ok(Leave::Return) => return_to_game as *const () as u32,
            Ok(Leave::Exit) => {
                // `find_thread_slot` found nothing: its null result is in esi.
                r.esi = 0;
                game_addr(EXIT_PATH_RVA) as u32
            }
            Ok(Leave::Routine { slot, routine }) => {
                r.esi = slot;
                routine
            }
            Err(_) => original,
        }
    };
    log(&format!(
        "fatal::report: the Rust version ran and continues at {target:#x} (the original reporter is at {original:#x})"
    ));

    // The slot the thunk reserved sits right above the saved registers and flags.
    unsafe { (r as *mut Regs as *mut u32).add(9).write(target) };
}

// ---- self-test: a copy of the original that cannot end the game ----

// pusha; pushf; pop edx; mov ebp, esp
// what the stand-in for find_thread_slot returns
// a slot: its first dword is the routine
