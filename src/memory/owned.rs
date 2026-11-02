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
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// `bytes` counted on `list` as `memory::allocate` counts a block (the list's total, the
/// memory figures). Refuses what `allocate` refuses: the game's own heap, no list, a size
/// that with the header wraps.
pub fn count_on(list: crate::pointers::GamePtr, bytes: u32) -> Result<(), Refused> {
    if own_heap() == 1 {
        return Err(BadInput("the game's own heap is in use (altmemenable)").into());
    }
    if list.is_null() || rc2k_core::memory::block_size(bytes).is_none() {
        return Err(BadInput("no list, or a size that wraps").into());
    }
    // As in `free_list`.
    let at = list.unbounded();
    let mut l = at.read::<MemoryList>("a memory list")?;
    l.total = l.total.wrapping_add(bytes);
    at.write(l, "a memory list")?;
    with_figures(|f| f.taken(bytes));
    Ok(())
}

/// `bytes` of Rust's memory, zeroed, counted on `list` as `memory::allocate` counts a
/// block (the list's `free_list` counts it down): a block of the game's heap that lives
/// only while one function runs. Sized as the game's block was (whole pages less its
/// header), so a read just past `bytes` gives zeros as it did. Refuses what `count_on`
/// refuses.
pub fn counted(list: crate::pointers::GamePtr, bytes: u32) -> Result<Vec<u8>, Refused> {
    count_on(list, bytes)?;
    Ok(vec![0; block_len(bytes)])
}

/// The bytes a block of the game's heap asked for `bytes` held: whole pages less its header.
pub fn block_len(bytes: u32) -> usize {
    let header = crate::memory::HEADER_LEN as usize;
    (bytes as usize + header).next_multiple_of(0x1000) - header
}

pub struct Owned<T> {
    value: Mutex<Option<Box<T>>>,
    /// The thread in a step on it (0: none). Only that thread's own check can match, so
    /// relaxed is enough; a second step on the same thread would otherwise wait for itself.
    holder: AtomicUsize,
    /// What it is, for the refusals.
    what: &'static str,
}

thread_local! {
    /// Only its address is used: what names this thread as a value's holder.
    static THREAD_MARK: u8 = const { 0 };
}

/// This thread's name, never 0.
#[inline]
fn this_thread() -> usize {
    THREAD_MARK.with(|m| std::ptr::from_ref::<u8>(m) as usize)
}

/// A step ended (also when it unwinds).
struct Held<'a>(&'a AtomicUsize);

impl Drop for Held<'_> {
    fn drop(&mut self) {
        self.0.store(0, Relaxed);
    }
}

impl<T> Owned<T> {
    pub const fn new(what: &'static str) -> Self {
        Self {
            value: Mutex::new(None),
            holder: AtomicUsize::new(0),
            what,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Box<T>>> {
        self.value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// `value` taken, `bytes` (the game's block size) counted on `list`.
    pub fn take(
        &self,
        list: crate::pointers::GamePtr,
        bytes: u32,
        value: T,
    ) -> Result<(), Refused> {
        count_on(list, bytes)?;
        *self.lock() = Some(Box::new(value));
        Ok(())
    }

    /// Given back (its list's total is counted down by `memory::free_list`).
    pub fn give_back(&self) {
        *self.lock() = None;
    }

    /// The value set without counting it: the 32-bit build's tests of code that reads it.
    #[cfg(all(test, target_arch = "x86"))]
    pub fn set(&self, value: T) {
        *self.lock() = Some(Box::new(value));
    }

    pub fn is_taken(&self) -> bool {
        self.lock().is_some()
    }

    /// `f` on the value. Refuses none taken and a step inside one on the same value.
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, BadInput> {
        let me = this_thread();
        if self.holder.load(Relaxed) == me {
            return Err(BadInput("a table reached again inside its own step"));
        }
        let mut value = self.lock();
        let value = value.as_mut().ok_or(BadInput(self.what))?;
        self.holder.store(me, Relaxed);
        let _held = Held(&self.holder);
        Ok(f(value))
    }
}

/// As `Owned`, for what the game's own thread alone reaches and reaches all the time (the course
/// and the objects, many times a physics tick): the value in that thread's storage, so a step
/// is a borrow flag and no lock. Another thread sees none taken, and is refused for it.
pub struct ThreadOwned<T: 'static> {
    value: &'static std::thread::LocalKey<std::cell::RefCell<Option<T>>>,
    what: &'static str,
}

impl<T> ThreadOwned<T> {
    pub const fn new(
        value: &'static std::thread::LocalKey<std::cell::RefCell<Option<T>>>,
        what: &'static str,
    ) -> Self {
        Self { value, what }
    }

    /// `value` taken, `bytes` (the game's block size) counted on `list`.
    pub fn take(
        &self,
        list: crate::pointers::GamePtr,
        bytes: u32,
        value: T,
    ) -> Result<(), Refused> {
        count_on(list, bytes)?;
        self.set(value);
        Ok(())
    }

    /// Given back (its list's total is counted down by `memory::free_list`).
    pub fn give_back(&self) {
        self.value.with(|v| *v.borrow_mut() = None);
    }

    /// The value set without counting it: tests of code that reads it, or a part moved out
    /// of a block that stays counted (`race::scene_points`).
    pub fn set(&self, value: T) {
        self.value.with(|v| *v.borrow_mut() = Some(value));
    }

    pub fn is_taken(&self) -> bool {
        self.value.with(|v| v.borrow().is_some())
    }

    /// `f` on the value. Refuses none taken and a step inside one on the same value.
    #[inline]
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, BadInput> {
        self.value.with(|v| {
            let mut held = v
                .try_borrow_mut()
                .map_err(|_| BadInput("a step inside a step on the same value"))?;
            match held.as_mut() {
                Some(value) => Ok(f(value)),
                None => Err(BadInput(self.what)),
            }
        })
    }
}

/// A `ThreadOwned` static and its thread's storage.
macro_rules! thread_owned {
    ($(#[$doc:meta])* $vis:vis static $name:ident: $t:ty = $what:expr;) => {
        $(#[$doc])*
        $vis static $name: $crate::memory::owned::ThreadOwned<$t> = {
            thread_local! {
                static VALUE: std::cell::RefCell<Option<$t>> =
                    const { std::cell::RefCell::new(None) };
            }
            $crate::memory::owned::ThreadOwned::new(&VALUE, $what)
        };
    };
}
pub(crate) use thread_owned;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taken_counted_lent_and_given_back() {
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let mut memory = vec![0u8; size_of::<MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        let l = crate::pointers::GamePtr::new(crate::pointers::game_address(memory.as_mut_ptr()));
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
