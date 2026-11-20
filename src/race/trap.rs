//! A refusal carried out of a closure: the core routines of `rc2k_core` ask the game's memory
//! through closures that cannot fail, and a checked read can be refused. The closure reads
//! through a `Trap`; the first refusal is kept (the closure answers 0 meanwhile, and the
//! caller discards that answer) and `finish` hands it back after the core returns.

use crate::errors::BadInput;
use std::cell::Cell;

#[derive(Default)]
pub struct Trap {
    refused: Cell<Option<BadInput>>,
}

impl Trap {
    pub fn new() -> Self {
        Self::default()
    }

    /// A checked read's answer, or its zero when refused (kept for `finish`).
    pub(super) fn keep_read<T: Default>(&self, read: Result<T, BadInput>) -> T {
        read.unwrap_or_else(|e| {
            self.keep(e);
            T::default()
        })
    }

    /// Keeps a refusal met elsewhere in the closure.
    pub fn keep(&self, e: BadInput) {
        if self.refused.get().is_none() {
            self.refused.set(Some(e));
        }
    }

    /// The first refusal, if any read was refused.
    pub fn finish(&self) -> Result<(), BadInput> {
        match self.refused.get() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}
