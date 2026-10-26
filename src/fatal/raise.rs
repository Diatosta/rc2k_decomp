//! Raising a fatal error from Rust: what the original's stubs do before they jump to the
//! reporter (store the code and the message), followed by the reporter itself.
//!
//! This is not a hook: nothing in the original corresponds to it one to one. It calls the
//! reporter's Rust version directly (`fatal::report::report_now`).
//!
//! It is not identical to an original error in one respect: the registers in the report are
//! the ones given here (those at the entry of the hooked function that failed), not those at
//! the instruction inside it where the original would have noticed.

use crate::errors::{FatalError, Refused};
use crate::fatal::state::{CODE, MESSAGE};
use crate::framework::{Regs, log};
use std::ffi::CString;

/// Reports `error` with the registers in `regs` and ends the game the way the original
/// does: on a thread without a thread slot through the exit path (which shows the report in
/// a message box), on a slot's thread by entering its routine again. It only returns for a
/// code without the fatal bit, which nothing in this build has.
pub unsafe fn raise(error: FatalError, regs: &Regs) {
    // The message has to outlive this call: the exit path shows it later. An error is raised
    // once, so the few bytes are simply never freed.
    let message = match CString::new(error.message) {
        Ok(text) if !error.message.is_empty() => Box::leak(text.into_boxed_c_str()).as_ptr() as u32,
        _ => 0,
    };
    CODE.store(error.code, std::sync::atomic::Ordering::Relaxed);
    MESSAGE.store(message, std::sync::atomic::Ordering::Relaxed);
    crate::fatal::report::report_now(regs);
}

/// What the program does when the Rust version `name` refuses a call it has nowhere to
/// hand on to. A refused input (`BadInput`) ends it, saying which function refused and
/// why (owner, 2026-10-23: no hidden fallbacks). A fatal error is raised from Rust (`raise`),
/// so that it shows: the game ends with the report, and the log says which function raised
/// it. If that ever happens where the original would have carried on, the Rust version is
/// wrong.
pub unsafe fn refused_in(name: &str, reason: impl Into<Refused>, r: &Regs) {
    let error = match reason.into() {
        Refused::BadInput(why) => {
            crate::standalone::finish(&format!("{name} refused: {}", why.0), 1)
        }
        Refused::Fatal(error) => error,
    };
    log(&format!(
        "{name}: the Rust version raises fatal error {:#x} {:?}",
        error.code, error.message
    ));
    unsafe { raise(error, r) };
}

/// The program ended for a refusal where no caller can carry it (a core's closure, a trait's
/// method, the installer's script): `refused_in` with no registers to report. A fatal error
/// raised this way that comes back (a code without the fatal bit) ends the program too.
pub fn end(name: &str, reason: impl Into<Refused>) -> ! {
    // A test sees the refusal as its own failure, not the whole run's end.
    #[cfg(test)]
    {
        panic!("{name}: refused: {:?}", reason.into())
    }
    #[cfg(not(test))]
    {
        unsafe { refused_in(name, reason, &Regs::default()) };
        crate::standalone::finish(
            &format!("{name}: a fatal error came back from its report"),
            1,
        )
    }
}
