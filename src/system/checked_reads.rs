//! Game reads asked by a converted core that cannot refuse (it takes `Fn(u32) -> u32`):
//! the first refusal is kept, the core runs on with a zero in its place, and the caller
//! raises the refusal before it uses the core's answer (`finish`). Nothing is returned that
//! was computed from a refused read.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use std::cell::Cell;

/// A refusal in a function whose callers have no way to carry it (a `bool`, an `Option`):
/// the program ends naming it, as a refusal does at the top (`GamePtr` logged the address).
pub fn refuse(e: BadInput) -> ! {
    crate::fatal::raise::end("a game read", e)
}

#[derive(Default)]
pub struct CheckedReads(Cell<Option<BadInput>>);

impl CheckedReads {
    pub fn new() -> Self {
        Self::default()
    }

    fn keep(&self, e: BadInput) {
        if self.0.get().is_none() {
            self.0.set(Some(e));
        }
    }

    /// The dword at `at`, or 0 with the refusal kept.
    pub fn dword(&self, at: u32, what: &str) -> u32 {
        GamePtr::new(at).read::<u32>(what).unwrap_or_else(|e| {
            self.keep(e);
            0
        })
    }

    /// A read's value, or 0 with the refusal kept.
    pub fn value(&self, read: Result<u32, BadInput>) -> u32 {
        read.unwrap_or_else(|e| {
            self.keep(e);
            0
        })
    }

    /// The core's `answer`, unless a read it asked for was refused.
    pub fn finish<T>(&self, answer: T) -> Result<T, BadInput> {
        self.0.take().map_or(Ok(answer), Err)
    }
}
