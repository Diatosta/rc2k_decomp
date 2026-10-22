//! Raising a fatal error from Rust: what the original's stubs do before they jump to the
//! reporter (store the code and the message), followed by the reporter itself.
//!
//! This is not a hook: nothing in the original corresponds to it one to one. It goes to the
//! reporter's address in the game, so it runs whatever is there: the `fatal::report` hook
//! (and through it the Rust version, or the original in shadow mode), or the original if the
//! hook is off. That also makes it the only way to send a real error through the hook's
//! hand-written thunk, which is what the console command `fatal` is for.
//!
//! It is not identical to an original error in one respect: the registers in the report are
//! the ones given here (those at the entry of the hooked function that failed), not those at
//! the instruction inside it where the original would have noticed.

use crate::errors::{FatalError, Refused};
use crate::fatal::report::{CODE_RVA, MESSAGE_RVA};
use crate::framework::{GameHook, RawHook, Regs, game_write, log, rva, shadow};
use crate::threading::find_thread_slot::find_thread_slot;
use std::ffi::CString;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

const REPORTER_RVA: usize = rva(0x40763C); // fatal::report

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
    unsafe {
        game_write(CODE_RVA, error.code);
        game_write(MESSAGE_RVA, message);
        let mut regs = *regs;
        crate::calls::call(REPORTER_RVA, &mut regs);
    }
}

/// What a hook does when its Rust version refuses a call.
///
/// A refused input (`BadInput`) goes to the original, as before.
/// A fatal error is raised from Rust (`raise`), so that it shows: the game ends with the
/// report, and hook.log says which hook raised it. If that ever happens where the original
/// would have carried on, the Rust version is wrong.
/// In shadow mode the original runs instead, as it does for every call there; if it comes
/// back, it did not meet the fatal error the Rust version reported, and that is logged as a
/// mismatch.
pub unsafe fn refused(
    hook: &RawHook,
    reason: impl Into<Refused>,
    r: &mut Regs,
    call_original: unsafe extern "C" fn(*mut Regs),
) {
    let error = match reason.into() {
        Refused::BadInput(why) => {
            if crate::framework::standalone() {
                crate::framework::log(&format!("{}: refused: {}", hook.name(), why.0));
            } else if first_refusal(hook.name(), why.0) {
                // Once a hook and reason: a refusal in play is a case the Rust version
                // leaves to the original, and should be known about.
                crate::framework::log(&format!(
                    "{}: refused (first time, the original runs): {}",
                    hook.name(),
                    why.0
                ));
            }
            unsafe { call_original(r) };
            return;
        }
        Refused::Fatal(error) => error,
    };
    if shadow() {
        unsafe { call_original(r) };
        hook.mismatch(|| {
            format!(
                "the Rust version reported fatal error {:#x} {:?}; the original returned",
                error.code, error.message
            )
        });
        return;
    }
    log(&format!(
        "{}: the Rust version raises fatal error {:#x} {:?}",
        hook.name(),
        error.code,
        error.message
    ));
    unsafe {
        raise(error, r);
        // Only a code without the fatal bit comes back, and no error has one.
        call_original(r);
    }
}

/// True the first time a hook refuses for a reason.
fn first_refusal(hook: &'static str, why: &'static str) -> bool {
    static SEEN: Mutex<Vec<(&'static str, &'static str)>> = Mutex::new(Vec::new());
    let mut seen = SEEN.lock().unwrap();
    if seen.contains(&(hook, why)) {
        return false;
    }
    seen.push((hook, why));
    true
}

// ---- the console command `fatal` ----

static TEST_ARMED: AtomicBool = AtomicBool::new(false);

/// What the test raises: no real code, and a message with a line break in it.
const TEST_ERROR: FatalError = FatalError {
    code: 0x8BAD_F00D,
    message: "rc2k_hook test error\\nraised by the console command `fatal`",
};

/// Makes the next `input::take_key` call raise `TEST_ERROR`. The game's own thread calls
/// that every frame; the console's thread must not raise it itself, because the exit path
/// would then close the game from the wrong thread.
pub fn arm_test() {
    TEST_ARMED.store(true, Ordering::SeqCst);
}

/// Called by the `input::take_key` hook.
pub fn raise_test_if_armed(regs: &Regs) {
    if !TEST_ARMED.swap(false, Ordering::SeqCst) {
        return;
    }
    let slot = unsafe { find_thread_slot() };
    log(&format!(
        "fatal test: raising {:#x} on the thread that asked for a key (thread slot {:?}); the game ends now",
        TEST_ERROR.code, slot
    ));
    unsafe { raise(TEST_ERROR, regs) };
}
