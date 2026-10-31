//! A refusal carried out of a closure: the core routines of `rc2k_core` ask the game's memory
//! through closures that cannot fail, and a `GamePtr` read can be refused. The closure reads
//! through a `Trap`; the first refusal is kept (the closure answers 0 meanwhile, and the
//! caller discards that answer) and `finish` hands it back after the core returns.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use std::cell::Cell;

#[derive(Default)]
pub struct Trap {
    refused: Cell<Option<BadInput>>,
}

impl Trap {
    pub fn new() -> Self {
        Self::default()
    }

    /// The `T` at `at`, or its zero when the read is refused (kept for `finish`).
    #[track_caller]
    pub fn read<T: Copy + Default>(&self, at: u32, what: &str) -> T {
        match GamePtr::new(at).read::<T>(what) {
            Ok(v) => v,
            Err(e) => {
                if self.refused.get().is_none() {
                    self.refused.set(Some(e));
                }
                T::default()
            }
        }
    }

    /// A checked read's answer, or its zero when refused (kept for `finish`).
    pub fn keep_read<T: Default>(&self, read: Result<T, BadInput>) -> T {
        read.unwrap_or_else(|e| {
            self.keep(e);
            T::default()
        })
    }

    /// As `read`, at a course place or a game address (`race::course_places::read`).
    pub fn course<T: crate::race::course_places::Whole + Default>(&self, at: u32, what: &str) -> T {
        match crate::race::course_places::read::<T>(at, what) {
            Ok(v) => v,
            Err(e) => {
                if self.refused.get().is_none() {
                    self.refused.set(Some(e));
                }
                T::default()
            }
        }
    }

    /// As `course`, a whole face (`race::course_places::face`).
    pub fn face(&self, at: u32, what: &str) -> rc2k_core::race::faces::Face {
        crate::race::course_places::face(at, what).unwrap_or_else(|e| {
            self.keep(e);
            Default::default()
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

/// Whether `len` bytes from `addr` lie in one of the game's regions, for the calls that still
/// take a raw pointer into the game's memory (file reads and writes): refused otherwise,
/// the address logged.
pub fn span(addr: u32, len: usize, what: &str) -> Result<(), BadInput> {
    match crate::pointers::region_of(addr) {
        Some((_, end)) if (end - addr) as usize >= len => Ok(()),
        _ => {
            crate::framework::log(&format!(
                "pointers: {what} at {addr:#010x} ({len} bytes) is outside every allocation"
            ));
            Err(BadInput(
                "a game pointer outside every allocation (rc2k.log names it)",
            ))
        }
    }
}

/// The car block at `addr` (a `g_carBlocks` entry, a view's block, `g_car`), checked to lie
/// in one region.
pub fn car_at(addr: u32) -> Result<&'static crate::physics::car_store::Car, BadInput> {
    GamePtr::new(addr).record::<crate::physics::car_store::Car>("a car block")
}

/// A test's own buffer made a region while the guard lives (any items; see
/// `pointers::TestRegion`).
#[cfg(all(test, target_arch = "x86"))]
pub fn test_region<T>(items: &[T]) -> crate::pointers::TestRegion {
    // SAFETY: the same memory seen as bytes, for the registry only.
    let bytes = unsafe {
        std::slice::from_raw_parts(items.as_ptr().cast::<u8>(), std::mem::size_of_val(items))
    };
    crate::pointers::TestRegion::new(bytes)
}
