//! A typed value Rust owns in place of a block of the game's heap (phase 12): the table is
//! a struct with named fields, not bytes at offsets. It is
//! taken as `memory::allocate` took the block (counted on the same list, so the game's
//! "MEM:" lines are unchanged) and given back with that list.
//!
//! Reached only through `Owned::with`, which lends it for one step: none taken is refused,
//! and so is a step that reaches the same value again inside its own step (it would wait
//! for itself). Another thread's step is waited for.

use crate::errors::{BadInput, Refused};
use crate::memory::{MemoryList, own_heap, with_figures};
use std::cell::RefCell;
use std::sync::Mutex;

/// `bytes` counted on `list` as `memory::allocate` counts a block (the list's total, the
/// memory figures). Refuses what `allocate` refuses: the game's own heap, no list, a size
/// that with the header wraps.
pub fn count_on(list: *mut MemoryList, bytes: u32) -> Result<(), Refused> {
    if own_heap() == 1 {
        return Err(BadInput("the game's own heap is in use (altmemenable)").into());
    }
    if list.is_null() || rc2k_core::memory::block_size(bytes).is_none() {
        return Err(BadInput("no list, or a size that wraps").into());
    }
    let at = crate::pointers::GamePtr::new(crate::pointers::game_address(list));
    let mut l = at.read::<MemoryList>("a memory list")?;
    l.total = l.total.wrapping_add(bytes);
    at.write(l, "a memory list")?;
    with_figures(|f| f.taken(bytes));
    Ok(())
}

pub struct Owned<T> {
    value: Mutex<Option<Box<T>>>,
    /// What it is, for the refusals.
    what: &'static str,
}

thread_local! {
    /// The values this thread is in a step on (by their `Owned`'s address).
    static HELD: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

/// A step ended (also when it unwinds).
struct Held(usize);

impl Drop for Held {
    fn drop(&mut self) {
        HELD.with(|h| h.borrow_mut().retain(|&b| b != self.0));
    }
}

impl<T> Owned<T> {
    pub const fn new(what: &'static str) -> Self {
        Self {
            value: Mutex::new(None),
            what,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Box<T>>> {
        self.value.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// `value` taken, `bytes` (the game's block size) counted on `list`.
    pub fn take(&self, list: *mut MemoryList, bytes: u32, value: T) -> Result<(), Refused> {
        count_on(list, bytes)?;
        *self.lock() = Some(Box::new(value));
        Ok(())
    }

    /// Given back (its list's total is counted down by `memory::free_list`).
    pub fn give_back(&self) {
        *self.lock() = None;
    }

    pub fn is_taken(&self) -> bool {
        self.lock().is_some()
    }

    /// `f` on the value. Refuses none taken and a step inside one on the same value.
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, BadInput> {
        let me = self as *const Self as usize;
        if HELD.with(|h| h.borrow().contains(&me)) {
            return Err(BadInput("a table reached again inside its own step"));
        }
        let mut value = self.lock();
        let value = value.as_mut().ok_or(BadInput(self.what))?;
        HELD.with(|h| h.borrow_mut().push(me));
        let _held = Held(me);
        Ok(f(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taken_counted_lent_and_given_back() {
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let mut memory = vec![0u8; size_of::<MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        let l = memory.as_mut_ptr() as *mut MemoryList;
        let owned: Owned<[u32; 2]> = Owned::new("no test table");
        assert_eq!(owned.with(|_| ()), Err(BadInput("no test table")));
        owned.take(l, 8, [1, 2]).unwrap();
        assert_eq!(u32::from_le_bytes(memory[4..8].try_into().unwrap()), 8);
        owned.with(|v| v[1] = 5).unwrap();
        assert_eq!(owned.with(|v| *v), Ok([1, 5]));
        assert!(owned.with(|_| owned.with(|_| ())).unwrap().is_err());
        owned.give_back();
        assert!(!owned.is_taken());
        crate::memory::with_figures(|f| *f = figures);
    }
}
