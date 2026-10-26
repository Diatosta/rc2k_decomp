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
use std::ffi::c_void;
use windows_sys::Win32::System::Threading::CreateThread;

// ---- slot layout ----

const ROUTINE: usize = 0;
const THREAD_ID: usize = 4;
const HANDLE: usize = 8;
const FLAG: usize = 0xC;
// ---- Rust version ----

/// The thread of the slot at game address `slot` started.
pub fn start_slot_thread(slot: u32) -> Result<(), FatalError> {
    let what = "a thread slot";
    crate::pointers::check_or_end(slot, 0x10, what);
    crate::pointers::write_or_end(slot + FLAG as u32, 0u8, what);
    // The slot holds the routine as the image did, an IDA address: the thread's
    // parameter, `run_routine` finds its Rust version by it.
    let routine = crate::pointers::read_or_end::<u32>(slot + ROUTINE as u32, what) as usize;
    // SAFETY: the thread's id goes straight into the slot (checked above), as the
    // original's `CreateThread` put it there before the thread could look for its slot.
    let handle = unsafe {
        CreateThread(
            std::ptr::null(),
            0,
            Some(run_routine),
            std::ptr::without_provenance::<c_void>(routine),
            0,
            crate::pointers::host(slot + THREAD_ID as u32).cast(),
        )
    };
    if handle.is_null() {
        return Err(FatalError {
            code: 0x8010_8601,
            message: "",
        });
    }
    crate::pointers::write_or_end(
        slot + HANDLE as u32,
        crate::pointers::handle_number(handle),
        what,
    );
    Ok(())
}

/// A slot's thread: its routine (the IDA address the slot holds) run by its Rust version
/// (`thread_slot::run_routine`). A routine with none, or one that refuses, ends the program.
extern "system" fn run_routine(routine: *mut c_void) -> u32 {
    // The parameter is the IDA address itself, no memory.
    let routine = routine.addr() as u32;
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
    0
}
