//! The blocks of bytes lent to the game at run time (the screen's and the textures' pixels,
//! the system's pages, the installer's and the course's buffers): each a store of its own,
//! reached through the place layer as the statics are, never by a raw pointer.
//!
//! A block is never freed. Given back, it waits in the pool for the next request it fits:
//! the smallest free block of at least the bytes asked for and at most twice as many (or a
//! page). Memory stays at the peak the game has reached (the menus' and the race's screens,
//! the largest stage's textures), and an address kept past its block's end can reach no
//! freed memory: its region is forgotten (`pointers::forget`), so a read of it is refused.

#![allow(unsafe_code)]

use crate::errors::BadInput;
use std::cell::UnsafeCell;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

const PAGE: usize = 0x1000;

/// A block of shared bytes, whole pages, page-aligned; lives for the program.
pub struct PoolBlock {
    bytes: &'static [UnsafeCell<u8>],
}

// SAFETY: the bytes are reached only as shared memory (`shared_memory`'s relaxed atomics,
// the discipline of every `Bytes`), or through the platform's slices under their promises.
unsafe impl Sync for PoolBlock {}
// SAFETY: as above.
unsafe impl Send for PoolBlock {}

impl PoolBlock {
    /// Its bytes (whole pages).
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Its first byte.
    pub fn as_ptr(&self) -> *mut u8 {
        UnsafeCell::raw_get(self.bytes.as_ptr())
    }

    /// The `len` bytes from `offset` inside it, or refused.
    fn check(&self, offset: usize, len: usize) -> Result<(), BadInput> {
        match offset.checked_add(len) {
            Some(end) if end <= self.len() => Ok(()),
            _ => Err(BadInput("a read or write past a lent block")),
        }
    }
}

// SAFETY: `cell` answers nothing: every read and write takes `load` and `store`, which reach
// the block's own shared bytes inside its length.
unsafe impl crate::pointers::Store for PoolBlock {
    fn size(&self) -> usize {
        self.len()
    }

    fn first(&self) -> *const u8 {
        self.as_ptr()
    }

    fn cell(&self, _offset: usize) -> Option<crate::pointers::CellAt> {
        None
    }

    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), BadInput> {
        self.check(offset, to.len())?;
        // SAFETY: inside the block (checked); `to` is Rust's.
        unsafe { crate::shared_memory::load(self.as_ptr().add(offset), to.as_mut_ptr(), to.len()) };
        Ok(())
    }

    fn store(&self, offset: usize, from: &[u8]) -> Result<(), BadInput> {
        self.check(offset, from.len())?;
        // SAFETY: as `load`'s.
        unsafe {
            crate::shared_memory::store(self.as_ptr().add(offset), from.as_ptr(), from.len());
        }
        Ok(())
    }
}

static FREE: Mutex<Vec<&'static PoolBlock>> = Mutex::new(Vec::new());
/// The blocks made and their bytes, for the log.
static MADE: AtomicUsize = AtomicUsize::new(0);
static MADE_BYTES: AtomicUsize = AtomicUsize::new(0);

/// A block of at least `len` bytes, zeroed: one of the pool's, or a new one. The program ends
/// when the system has no memory for a new one (as `Vec` does).
pub fn take(len: usize) -> &'static PoolBlock {
    let need = len.max(1).next_multiple_of(PAGE);
    let reused = {
        let mut free = FREE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let best = free
            .iter()
            .enumerate()
            .filter(|(_, b)| b.len() >= need && b.len() <= need.saturating_mul(2).max(PAGE))
            .min_by_key(|(_, b)| b.len())
            .map(|(n, _)| n);
        best.map(|n| free.swap_remove(n))
    };
    if let Some(block) = reused {
        // SAFETY: the block's own bytes; it is lent to no one while in the pool.
        unsafe { crate::shared_memory::fill(block.as_ptr(), 0, need) };
        return block;
    }
    let layout = std::alloc::Layout::from_size_align(need, PAGE).unwrap_or_else(|_| {
        crate::fatal::raise::end("pointers::pool", BadInput("a block too large"))
    });
    // SAFETY: a layout of a non-zero size.
    let first = unsafe { std::alloc::alloc_zeroed(layout) };
    if first.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    // SAFETY: `need` zeroed bytes at `first`, never freed (the block lives for the program);
    // `UnsafeCell<u8>` has `u8`'s layout.
    let bytes = unsafe { std::slice::from_raw_parts(first.cast::<UnsafeCell<u8>>(), need) };
    let blocks = MADE.fetch_add(1, Relaxed) + 1;
    let total = MADE_BYTES.fetch_add(need, Relaxed) + need;
    crate::framework::log(&format!(
        "pointers: the pool's blocks grow to {blocks}, {} KiB (one of {} KiB made)",
        total / 1024,
        need / 1024
    ));
    Box::leak(Box::new(PoolBlock { bytes }))
}

/// A block back in the pool, once nothing reaches it (its region forgotten).
pub fn give_back(block: &'static PoolBlock) {
    FREE.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(block);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointers::Store;

    #[test]
    fn blocks_are_zeroed_reused_and_bounded() {
        let a = take(5000);
        assert_eq!(a.len(), 0x2000, "whole pages");
        assert_eq!(a.as_ptr() as usize % PAGE, 0, "page-aligned");
        a.store(8, &[7, 7]).unwrap();
        assert!(a.store(0x1FFF, &[1, 2]).is_err(), "past its end");
        give_back(a);
        // The same size again: a free block of the pool, zeroed (another test may take it
        // first: then a new one, zeroed as well).
        let b = take(5000);
        let mut two = [9u8; 2];
        b.load(8, &mut two).unwrap();
        assert_eq!(two, [0, 0]);
        give_back(b);
        // Far smaller: not served by a block more than twice its size.
        let big = take(4 * PAGE);
        let big_first = big.as_ptr();
        give_back(big);
        let c = take(0x10);
        assert!(c.len() <= 2 * PAGE);
        assert_ne!(c.as_ptr(), big_first);
        give_back(c);
    }
}
