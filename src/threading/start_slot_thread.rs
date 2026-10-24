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

pub unsafe fn start_slot_thread(slot: *mut u8) -> Result<(), FatalError> {
    unsafe {
        slot.add(FLAG).write(0);
        // The slot holds the routine as the image did, an IDA address.
        let routine = (slot.add(ROUTINE) as *const u32).read_unaligned() as usize;
        let routine = crate::framework::game_addr(crate::framework::rva(routine));
        let handle = CreateThread(
            std::ptr::null(),
            0,
            Some(run_routine),
            routine as *const c_void,
            0,
            slot.add(THREAD_ID).cast(),
        );
        if handle.is_null() {
            return Err(FatalError {
                code: 0x8010_8601,
                message: "",
            });
        }
        (slot.add(HANDLE) as *mut u32).write_unaligned(handle as u32);
    }
    Ok(())
}

/// A slot's thread: its routine (the game's address, as the slot holds it) run by its Rust
/// version (`crate::routes`). A routine with none, or one that refuses, ends the program.
unsafe extern "system" fn run_routine(routine: *mut c_void) -> u32 {
    let routine = routine as usize;
    let mut r = Regs::default();
    let at = routine.wrapping_sub(crate::framework::game_addr(0));
    let ran = { crate::routes::by_rva(at, &mut r) };
    match ran {
        Some(Ok(())) => {}
        Some(Err(crate::errors::Refused::Fatal(error))) => {
            log(&format!(
                "threading::start_slot_thread: the routine {routine:#x} raises fatal error {:#x} {:?}",
                error.code, error.message
            ));
            unsafe { crate::fatal::raise::raise(error, &r) };
        }
        Some(Err(crate::errors::Refused::BadInput(why))) => crate::standalone::finish(
            &format!(
                "threading::start_slot_thread: the routine {:#x} refused: {}",
                at + crate::framework::IMAGE_BASE,
                why.0
            ),
            1,
        ),
        None => crate::standalone::not_ported(
            at + crate::framework::IMAGE_BASE,
            "threading::start_slot_thread",
        ),
    }
    0
}
