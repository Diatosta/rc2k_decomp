//! The blocks of bytes the platform makes at run time (the screen's and the textures'
//! pixels, `PoolBytes`): each a store of its own, read and written inside its length.
//!
//! A block is pages mapped from the system (`os::map_pages`) and is never unmapped. Given
//! back, its pages are handed back to the system (`os::discard_pages`: still mapped, their
//! memory freed) and it waits in the pool for the next request it fits: the smallest free
//! block of at least the bytes asked for and at most twice as many (or a page). So the
//! program holds the memory of what is held now, untouched pages cost nothing (textures
//! made and never filled), and a read of a block already given back meets the pages zeroed
//! or discarded, never a fault.

#![allow(unsafe_code)]

use crate::errors::BadInput;
use crate::memory::cells::Plain;
use crate::program_log::log;
use std::cell::UnsafeCell;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

const PAGE: usize = 0x1000;

/// A block of shared bytes, whole pages, page-aligned; lives for the program.
pub struct PoolBlock {
    bytes: &'static [UnsafeCell<u8>],
}

// SAFETY: the bytes are reached only as shared memory (`memory::shared_memory`'s word copies, one
// width per byte), or through the platform's slices under their promises.
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

// SAFETY: `first` and `size` are the block's own pages; every read and write takes `load` and
// `store`, which reach them inside its length.
unsafe impl crate::memory::cells::Store for PoolBlock {
    fn size(&self) -> usize {
        self.len()
    }

    fn first(&self) -> *const u8 {
        self.as_ptr()
    }

    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), BadInput> {
        self.check(offset, to.len())?;
        // SAFETY: inside the block (checked), whole pages so 4-aligned, reached only by word
        // copies; `to` is Rust's.
        unsafe { crate::memory::shared_memory::load_words(self.as_ptr(), self.len(), offset, to) };
        Ok(())
    }

    fn store(&self, offset: usize, from: &[u8]) -> Result<(), BadInput> {
        self.check(offset, from.len())?;
        // SAFETY: as `load`'s.
        unsafe {
            crate::memory::shared_memory::store_words(self.as_ptr(), self.len(), offset, from);
        }
        Ok(())
    }
}

/// The free blocks, with whether their pages came back zeroed from the system.
static FREE: Mutex<Vec<(&'static PoolBlock, bool)>> = Mutex::new(Vec::new());
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
            .filter(|(_, (b, _))| b.len() >= need && b.len() <= need.saturating_mul(2).max(PAGE))
            .min_by_key(|(_, (b, _))| b.len())
            .map(|(n, _)| n);
        best.map(|n| free.swap_remove(n))
    };
    if let Some((block, zeroed)) = reused {
        if !zeroed {
            for at in (0..block.len()).step_by(PAGE) {
                let _ = crate::memory::cells::Store::store(block, at, &[0; PAGE]);
            }
        }
        return block;
    }
    let first = crate::platform::os::map_pages(need).unwrap_or_else(|| {
        crate::fatal::raise::end("pool", BadInput("no memory for a lent block"))
    });
    // SAFETY: `need` zeroed bytes mapped at `first`, never unmapped (the block lives for the
    // program); `UnsafeCell<u8>` has `u8`'s layout.
    let bytes = unsafe { std::slice::from_raw_parts(first.cast::<UnsafeCell<u8>>(), need) };
    let blocks = MADE.fetch_add(1, Relaxed) + 1;
    let total = MADE_BYTES.fetch_add(need, Relaxed) + need;
    // Said at each doubling of the count: a start makes several hundred blocks (one for each
    // texture), and a line for each made the log hard to read.
    if blocks.is_power_of_two() {
        log(&format!(
            "pointers: the pool maps {blocks} blocks, {} KiB of addresses (the last made of {} \
             KiB; a free block keeps its addresses, not its memory)",
            total / 1024,
            need / 1024
        ));
    }
    Box::leak(Box::new(PoolBlock { bytes }))
}

/// A block back in the pool, once nothing reaches it: its pages handed back to the system
/// first.
pub fn give_back(block: &'static PoolBlock) {
    // SAFETY: the block is one `map_pages` mapping of whole pages.
    let zeroed = unsafe { crate::platform::os::discard_pages(block.as_ptr(), block.len()) };
    FREE.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((block, zeroed));
}

/// Bytes owned here, for reading and for writing, in a block of the pool: the screen's and
/// the textures' pixels, named by handle (`gfx::locked_pixels`) and shared with the drawing
/// threads. Every read and write is checked against their length. They are held by a raw
/// pointer, so a write through `as_mut_ptr` is allowed (a `Vec` lent through `&[u8]` gives
/// read permission only, which Miri rejects).
pub struct PoolBytes {
    block: &'static PoolBlock,
    len: usize,
    /// The rows written since they were last taken, for bytes made with `noting_rows` (the
    /// screen's): what the GPU's path uploads of the canvas, in place of looking at every row.
    rows: Option<WrittenRows>,
    /// Called before every write through `write_at` / `write_value` (`with_write_hook`).
    before_write: Option<fn()>,
}

/// One flag per row of `pitch` bytes: set by every write through `write_at` / `write_value`
/// that reaches the row.
struct WrittenRows {
    pitch: usize,
    flags: Box<[AtomicBool]>,
}

impl WrittenRows {
    fn note(&self, offset: usize, len: usize) {
        let Some(last_byte) = len.checked_sub(1).map(|l| offset + l) else {
            return;
        };
        let last = (last_byte / self.pitch).min(self.flags.len().saturating_sub(1));
        for flag in self.flags.get(offset / self.pitch..=last).unwrap_or(&[]) {
            flag.store(true, Relaxed);
        }
    }
}

impl PoolBytes {
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        let block = take(bytes.len());
        crate::memory::cells::Store::store(block, 0, &bytes)
            .unwrap_or_else(|e| crate::fatal::raise::end("pool::PoolBytes", e));
        Self {
            block,
            len: bytes.len(),
            rows: None,
            before_write: None,
        }
    }

    /// The same bytes, `hook` called before each write through `write_at` or `write_value` (a
    /// write in place through `as_mut_ptr` does not call it). It may itself write.
    pub(super) fn with_write_hook(mut self, hook: fn()) -> Self {
        self.before_write = Some(hook);
        self
    }

    /// As `from_vec`, the rows of `pitch` bytes written noted (`take_written_rows`), all of
    /// them written to begin with.
    pub(super) fn noting_rows(bytes: Vec<u8>, pitch: usize) -> Self {
        let rows = bytes.len() / pitch.max(1);
        let mut noting = Self::from_vec(bytes);
        noting.rows = Some(WrittenRows {
            pitch: pitch.max(1),
            flags: (0..rows).map(|_| AtomicBool::new(true)).collect(),
        });
        noting
    }

    /// The rows written since the last call (all of them after `note_all_rows_written`), one
    /// flag a row, each cleared; `None` for bytes not noting rows, or rows of another pitch.
    pub fn take_written_rows(&self, pitch: usize) -> Option<Vec<bool>> {
        let rows = self.rows.as_ref().filter(|r| r.pitch == pitch)?;
        Some(rows.flags.iter().map(|f| f.swap(false, Relaxed)).collect())
    }

    /// Whether any row is noted as written, the notes left as they are; `None` for bytes not
    /// noting rows, or rows of another pitch.
    pub fn any_row_written(&self, pitch: usize) -> Option<bool> {
        let rows = self.rows.as_ref().filter(|r| r.pitch == pitch)?;
        Some(rows.flags.iter().any(|f| f.load(Relaxed)))
    }

    /// Every row noted as written: for writes made in place (`as_mut_ptr`), which do not note.
    pub fn note_all_rows_written(&self) {
        if let Some(rows) = &self.rows {
            rows.flags.iter().for_each(|f| f.store(true, Relaxed));
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// `len` bytes from byte `offset`; a read past the bytes' end is refused.
    pub fn read_at(&self, offset: u32, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        let end = (offset as usize)
            .checked_add(len)
            .filter(|&e| e <= self.len);
        if end.is_none() {
            log(&format!(
                "pointers: {what}: {len} bytes at offset {offset:#x}, past the {:#x} bytes",
                self.len
            ));
            return Err(BadInput("past the end of a field"));
        }
        let mut bytes = vec![0; len];
        crate::memory::cells::Store::load(self.block, offset as usize, &mut bytes)?;
        Ok(bytes)
    }

    /// `data` written from byte `offset`; a write past the bytes' end is refused.
    pub fn write_at(&self, offset: u32, data: &[u8], what: &str) -> Result<(), BadInput> {
        let end = (offset as usize)
            .checked_add(data.len())
            .filter(|&e| e <= self.len);
        if end.is_none() {
            log(&format!(
                "pointers: {what}: {} bytes at offset {offset:#x}, past the {:#x} bytes",
                data.len(),
                self.len
            ));
            return Err(BadInput("past the end of a field"));
        }
        if let Some(hook) = self.before_write {
            hook();
        }
        if let Some(rows) = &self.rows {
            rows.note(offset as usize, data.len());
        }
        crate::memory::cells::Store::store(self.block, offset as usize, data)
    }

    /// The `T` at byte `offset`; a read past the bytes' end is refused.
    pub fn read_value<T: Plain>(&self, offset: u32, what: &str) -> Result<T, BadInput> {
        self.within(offset, size_of::<T>(), what)?;
        let mut value = crate::memory::cells::zeroed::<T>();
        crate::memory::cells::Store::load(
            self.block,
            offset as usize,
            crate::memory::cells::bytes_mut(&mut value),
        )?;
        Ok(value)
    }

    /// `value` written at byte `offset`; a write past the bytes' end is refused.
    pub fn write_value<T: Plain>(&self, offset: u32, value: T, what: &str) -> Result<(), BadInput> {
        self.within(offset, size_of::<T>(), what)?;
        if let Some(hook) = self.before_write {
            hook();
        }
        if let Some(rows) = &self.rows {
            rows.note(offset as usize, size_of::<T>());
        }
        crate::memory::cells::Store::store(
            self.block,
            offset as usize,
            crate::memory::cells::bytes_of(&value),
        )
    }

    /// Whether `len` bytes from `offset` lie inside the bytes.
    fn within(&self, offset: u32, len: usize, what: &str) -> Result<(), BadInput> {
        if (offset as usize)
            .checked_add(len)
            .is_some_and(|e| e <= self.len)
        {
            return Ok(());
        }
        log(&format!(
            "pointers: {what}: {len} bytes at offset {offset:#x}, past the {:#x} bytes",
            self.len
        ));
        Err(BadInput("past the end of a field"))
    }

    /// A copy of the bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = vec![0; self.len];
        crate::memory::cells::Store::load(self.block, 0, &mut bytes)
            .unwrap_or_else(|e| crate::fatal::raise::end("pool::PoolBytes", e));
        bytes
    }

    /// The bytes' first byte, for a slice of them written in place.
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.block.as_ptr()
    }

    /// The bytes in place.
    ///
    /// # Safety
    /// Nothing may write them while the slice lives.
    pub unsafe fn slice(&self) -> &[u8] {
        // SAFETY: the block holds `len` bytes; unwritten meanwhile (the caller's promise).
        unsafe { std::slice::from_raw_parts(self.block.as_ptr(), self.len) }
    }
}

impl Drop for PoolBytes {
    fn drop(&mut self) {
        give_back(self.block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::cells::Store;

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

    #[test]
    fn written_rows_are_noted_and_taken() {
        let bytes = PoolBytes::noting_rows(vec![0; 40], 10);
        assert_eq!(
            bytes.take_written_rows(10),
            Some(vec![true; 4]),
            "all at first"
        );
        assert_eq!(bytes.take_written_rows(10), Some(vec![false; 4]), "cleared");
        assert_eq!(bytes.take_written_rows(8), None, "another pitch");
        bytes.write_value(9u32, 7, "test").unwrap();
        bytes.write_at(35, &[1, 2, 3], "test").unwrap();
        bytes.write_at(20, &[], "test").unwrap();
        assert!(bytes.write_at(39, &[1, 2], "test").is_err());
        assert_eq!(
            bytes.take_written_rows(10),
            Some(vec![true, true, false, true])
        );
        bytes.note_all_rows_written();
        assert_eq!(bytes.take_written_rows(10), Some(vec![true; 4]));
        let plain = PoolBytes::from_vec(vec![0; 40]);
        plain.write_at(0, &[1], "test").unwrap();
        assert_eq!(plain.take_written_rows(10), None);
    }
}
