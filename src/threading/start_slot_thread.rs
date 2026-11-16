//! Starts the thread of a thread slot: the slot's flag cleared, a thread made (named for the
//! slot) that runs the slot's routine, the thread's id and handle kept in the slot. The thread
//! works with the owners it is given. A thread that cannot be made is fatal error 0x80108601.

use crate::errors::FatalError;
use crate::logfile::program_log::log;
use crate::threading::thread_slot::SlotOwners;

/// The thread of slot number `slot` started.
pub fn start_slot_thread(slot: usize, owners: SlotOwners<'static>) -> Result<(), FatalError> {
    let what = "a thread slot";
    let name = crate::threading::thread_slot::NAMES
        .get(slot)
        .copied()
        .unwrap_or("slot");
    let slot = crate::threading::thread_slot::slot_ref(slot)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
    slot.flag.set(0);
    // The slot holds the routine by its game address: the thread's parameter; `run_routine`
    // finds its Rust function by it.
    let routine = slot.routine.get() as usize;
    // The thread's id goes into the slot before the thread can look for its slot.
    let id = crate::platform::sync::new_thread_id();
    slot.thread_id.set(id);
    let Some(handle) = crate::platform::sync::spawn(name, id, move || {
        run_routine(routine as u32, owners);
    }) else {
        return Err(FatalError {
            code: 0x8010_8601,
            message: "",
        });
    };
    slot.handle.set(handle);
    Ok(())
}

/// A slot's thread: its routine run by its Rust function (`thread_slot::run_routine`). A
/// routine with none, or one that refuses, ends the program.
fn run_routine(routine: u32, owners: SlotOwners<'static>) {
    match crate::threading::thread_slot::run_routine(routine, owners) {
        Some(Ok(())) => {}
        Some(Err(crate::errors::Refused::Fatal(error))) => {
            log(&format!(
                "threading::start_slot_thread: the routine {routine:#x} raises fatal error {:#x} {:?}",
                error.code, error.message
            ));
            crate::fatal::raise::raise_bare(error);
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
