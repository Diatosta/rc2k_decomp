//! `threading::start_slot_thread` (0x410C9C): starts the thread of a thread slot.
//!
//! In:  esi = the slot
//! Out: every register preserved
//! It clears the slot's flag byte at +0xC, creates a thread running the routine at +0 (no
//! parameter, default stack), which stores the thread's id at +4, and stores the handle at
//! +8. If no thread can be created the original reports fatal error 0x80108601.
//! Four callers: three in `sub_410B84` and `request_quit`.
//! It creates a real thread, so shadow mode runs the original alone. The self-test uses
//! slots of its own whose routine returns at once.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::FatalError;
use crate::framework::{Regs, log};

// ---- Rust version ----

/// The thread of slot number `slot` started.
pub fn start_slot_thread(slot: usize) -> Result<(), FatalError> {
    let what = "a thread slot";
    let slot = crate::threading::thread_slot::slot_ref(slot)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
    slot.flag.set(0);
    // The slot holds the routine as the image did, an IDA address: the thread's
    // parameter, `run_routine` finds its Rust version by it.
    let routine = slot.routine.get() as usize;
    // The thread's id goes into the slot before the thread can look for its slot, as
    // the original's `CreateThread` put it there.
    let id = crate::sync::new_thread_id();
    slot.thread_id.set(id);
    let Some(handle) = crate::sync::spawn(&format!("slot {routine:#x}"), id, move || {
        run_routine(routine as u32);
    }) else {
        return Err(FatalError {
            code: 0x8010_8601,
            message: "",
        });
    };
    slot.handle.set(handle);
    Ok(())
}

/// A slot's thread: its routine (the IDA address the slot holds) run by its Rust version
/// (`thread_slot::run_routine`). A routine with none, or one that refuses, ends the program.
fn run_routine(routine: u32) {
    match crate::threading::thread_slot::run_routine(routine) {
        Some(Ok(())) => {}
        Some(Err(crate::errors::Refused::Fatal(error))) => {
            log(&format!(
                "threading::start_slot_thread: the routine {routine:#x} raises fatal error {:#x} {:?}",
                error.code, error.message
            ));
            crate::fatal::raise::raise(error, &Regs::default());
        }
        Some(Err(crate::errors::Refused::BadInput(why))) => crate::standalone::finish(
            &format!(
                "threading::start_slot_thread: the routine {routine:#x} refused: {}",
                why.0
            ),
            1,
        ),
        None => crate::standalone::not_ported(routine as usize, "threading::start_slot_thread"),
    }
}
