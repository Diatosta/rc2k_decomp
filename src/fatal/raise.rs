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

/// Reports `error` with the registers in `regs` and ends the game the way the original
/// does: on a thread without a thread slot through the exit path (which shows the report in
/// a message box), on a slot's thread by entering its routine again. It only returns for a
/// code without the fatal bit, which nothing in this build has.
pub fn raise(error: FatalError, regs: &Regs) {
    // None for an empty one, and for one with a NUL inside (a C string could not hold it).
    let message =
        (!error.message.is_empty() && !error.message.contains('\0')).then_some(error.message);
    CODE.store(error.code, std::sync::atomic::Ordering::Relaxed);
    *MESSAGE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = message;
    crate::fatal::report::report_now(regs);
}

/// What the program does when the Rust version `name` refuses a call it has nowhere to
/// hand on to. A refused input (`BadInput`) ends it, saying which function refused and
/// why (owner, 2026-10-23: no hidden fallbacks). A fatal error is raised from Rust (`raise`),
/// so that it shows: the game ends with the report, and the log says which function raised
/// it. If that ever happens where the original would have carried on, the Rust version is
/// wrong.
pub fn refused_in(name: &str, reason: impl Into<Refused>, r: &Regs) {
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
    raise(error, r);
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
        refused_in(name, reason, &Regs::default());
        crate::standalone::finish(
            &format!("{name}: a fatal error came back from its report"),
            1,
        )
    }
}
