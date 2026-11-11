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

    /// The `T` at byte `offset` of a store, or its zero when the read is refused (kept for
    /// `finish`).
    #[track_caller]
    pub fn load_at<T: crate::pointers::Plain + Default>(
        &self,
        store: &dyn crate::pointers::Store,
        offset: usize,
    ) -> T {
        self.keep_read(crate::pointers::load_value(store, offset))
    }

    /// A checked read's answer, or its zero when refused (kept for `finish`).
    pub fn keep_read<T: Default>(&self, read: Result<T, BadInput>) -> T {
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

/// The car block at `addr` (a `g_carBlocks` entry, a view's block, `g_car`: the work car or
/// one of the ten blocks); any other address is refused (the tests' own cars are found among
/// their regions).
pub fn car_at(addr: u32) -> Result<&'static crate::physics::car_store::Car, BadInput> {
    match crate::physics::car_store::Car::known(addr) {
        Some(car) => Ok(car),
        #[cfg(test)]
        None => crate::pointers::GamePtr::new(addr)
            .record::<crate::physics::car_store::Car>("a car block"),
        #[cfg(not(test))]
        None => {
            crate::framework::log(&format!("trap: {addr:#x} is no car block's place"));
            Err(BadInput("a car block at no car's place"))
        }
    }
}

/// A test's own buffer made a region while the guard lives (any items; see
/// `pointers::TestRegion`).
#[cfg(all(test, target_arch = "x86"))]
pub fn test_region<T>(items: &[T]) -> crate::pointers::Lent {
    // SAFETY: the same memory seen as bytes, for the registry only.
    let bytes = unsafe {
        std::slice::from_raw_parts(items.as_ptr().cast::<u8>(), std::mem::size_of_val(items))
    };
    crate::pointers::Lent::new(bytes)
}
