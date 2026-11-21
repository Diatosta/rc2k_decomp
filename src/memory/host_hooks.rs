//! What the memory layer asks of the program around it: a line for the program's log and the
//! ending of the program for a refusal no caller can carry. The program registers both once at
//! its start; until then (and in tests) log lines are dropped and a refusal panics.

use crate::errors::BadInput;
use std::sync::OnceLock;

/// The two functions the program registers.
pub struct Hooks {
    /// Writes one line to the program's log.
    pub log: fn(&str),
    /// Ends the program for a refusal (the name says who refused).
    pub end: fn(&str, BadInput) -> !,
}

static HOOKS: OnceLock<Hooks> = OnceLock::new();

/// Registers the hooks; the first call counts.
pub fn set(hooks: Hooks) {
    let _ = HOOKS.set(hooks);
}

/// One line to the program's log (dropped while no hooks are registered).
pub fn log(line: &str) {
    if let Some(hooks) = HOOKS.get() {
        (hooks.log)(line);
    }
}

/// The program ended for a refusal no caller can carry.
pub fn end(name: &str, reason: BadInput) -> ! {
    match HOOKS.get() {
        Some(hooks) => (hooks.end)(name, reason),
        None => panic!("{name}: refused: {reason:?}"),
    }
}
