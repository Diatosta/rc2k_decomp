//! How the platform's code ends the program and reports a fault without naming the code above
//! it: the program registers its two handlers once at its start (`set_handlers`), and the
//! threads' and the system's handlers call them through here. Before any registration (a unit
//! test, a thread started in a library's own use) the program ends plainly.

use std::sync::OnceLock;

/// What the program does where the platform cannot go on.
#[derive(Clone, Copy)]
pub struct Handlers {
    /// Ends the program with the reason and the exit code.
    pub finish: fn(&str, u32) -> !,
    /// Reports an access violation and ends the program: the faulting instruction's address, the
    /// address read or written, whether it was a write, and the stack pointer.
    pub report_fault: fn(usize, usize, bool, usize) -> !,
}

static HANDLERS: OnceLock<Handlers> = OnceLock::new();

/// Registers the program's handlers, once, at its start.
pub fn set_handlers(handlers: Handlers) {
    let _ = HANDLERS.set(handlers);
}

/// Ends the program with `why` and the exit `code`.
pub fn finish(why: &str, code: u32) -> ! {
    match HANDLERS.get() {
        Some(handlers) => (handlers.finish)(why, code),
        None => {
            eprintln!("rc2k: ending: {why}");
            std::process::exit(code as i32)
        }
    }
}

/// Reports an access violation, from the system's fault handler, and ends the program.
pub fn report_fault(host: usize, data: usize, write: bool, sp: usize) -> ! {
    match HANDLERS.get() {
        Some(handlers) => (handlers.report_fault)(host, data, write, sp),
        None => std::process::abort(),
    }
}
