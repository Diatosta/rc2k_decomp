//! The game's heap blocks as Rust's own memory (phase 12): what `memory::allocate` gave as
//! an address in the game's address space is here a `Vec` named by an id (its place + 1, 0
//! none), reached through `At` (a block and an offset) and never by address.
//!
//! A block is counted on its list as `allocate` counts one (the list's total, the memory
//! figures), so the game's "MEM:" lines are unchanged, and goes with the list
//! (`memory::free_list`). Its memory is sized as the game's was (whole pages less the
//! block's header): a read just past what was asked for reads zeros, as in the game, and is
//! not refused. Ids are not used again, so a stale one (a block given back) is refused.

use crate::errors::{BadInput, Refused};
use crate::memory::{HEADER_LEN, MemoryList, own_heap, with_figures};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

struct Block {
    /// The list it is counted on (the list's identity, never read through).
    list: usize,
    /// Its own lock: a step that reaches one block inside another's works; one that
    /// reaches the same block again is refused (`At::with_bytes`), not left waiting.
    bytes: Arc<Mutex<Vec<u8>>>,
}

static BLOCKS: Mutex<Vec<Option<Block>>> = Mutex::new(Vec::new());

fn blocks() -> std::sync::MutexGuard<'static, Vec<Option<Block>>> {
    BLOCKS.lock().unwrap_or_else(|e| e.into_inner())
}

thread_local! {
    /// The blocks this thread is in a step on (`At::with_bytes`).
    static HELD: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

/// A block's step ended (also when it unwinds).
struct Held(u32);

impl Drop for Held {
    fn drop(&mut self) {
        HELD.with(|h| h.borrow_mut().retain(|&b| b != self.0));
    }
}

/// The bytes a block of `bytes` gives: its whole pages less the header.
fn block_bytes(bytes: u32) -> usize {
    (bytes as usize + HEADER_LEN as usize).next_multiple_of(0x1000) - HEADER_LEN as usize
}

/// `memory::allocate` for an owned block: `bytes` zeroed, counted on `list`; its id.
/// Refuses what `allocate` refuses: the game's own heap, no list, a size that with the
/// header wraps.
pub fn take(list: *mut MemoryList, bytes: u32) -> Result<u32, Refused> {
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
    let mut all = blocks();
    all.push(Some(Block {
        list: list as usize,
        bytes: Arc::new(Mutex::new(vec![0; block_bytes(bytes)])),
    }));
    Ok(all.len() as u32)
}

/// Every block on `list` given back (`memory::free_list`, which counts the figures down).
pub fn free_on(list: *mut MemoryList) {
    for slot in blocks().iter_mut() {
        if slot.as_ref().is_some_and(|b| b.list == list as usize) {
            *slot = None;
        }
    }
}

/// A value kept in a block as the game's little-endian bytes.
pub trait Plain: Copy {
    const LEN: usize;
    fn from_bytes(b: &[u8]) -> Self;
    fn to_bytes(self, b: &mut [u8]);
}

macro_rules! plain {
    ($($t:ty),*) => {$(
        impl Plain for $t {
            const LEN: usize = size_of::<$t>();
            fn from_bytes(b: &[u8]) -> Self {
                <$t>::from_le_bytes(b.try_into().expect("Plain::LEN bytes"))
            }
            fn to_bytes(self, b: &mut [u8]) {
                b.copy_from_slice(&self.to_le_bytes());
            }
        }
    )*};
}
plain!(u8, i8, u16, i16, u32, i32, f32);

/// A place in an owned block: the block's id and an offset in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct At {
    pub block: u32,
    pub offset: u32,
}

impl At {
    /// The start of block `block`.
    pub const fn start(block: u32) -> Self {
        Self { block, offset: 0 }
    }

    /// `n` bytes further on.
    pub const fn add(self, n: u32) -> Self {
        Self {
            block: self.block,
            offset: self.offset.wrapping_add(n),
        }
    }

    /// `f` on the `len` bytes from here. Refuses a block that is not there (none, given
    /// back) and bytes past its memory.
    pub fn with_bytes<R>(
        self,
        len: usize,
        what: &'static str,
        f: impl FnOnce(&mut [u8]) -> R,
    ) -> Result<R, BadInput> {
        let block = (self.block as usize)
            .checked_sub(1)
            .and_then(|n| blocks().get(n)?.as_ref().map(|b| Arc::clone(&b.bytes)))
            .ok_or(BadInput("a block that is not there"))?;
        if HELD.with(|h| h.borrow().contains(&self.block)) {
            return Err(BadInput("a block reached again inside its own step"));
        }
        // Another thread's step on it is waited for.
        let mut bytes = block.lock().unwrap_or_else(|e| e.into_inner());
        HELD.with(|h| h.borrow_mut().push(self.block));
        let _held = Held(self.block);
        let start = self.offset as usize;
        let size = bytes.len();
        match start.checked_add(len).filter(|&end| end <= size) {
            Some(end) => Ok(f(&mut bytes[start..end])),
            None => {
                crate::framework::log(&format!(
                    "memory::blocks: {what} at {start:#x} ({len} bytes) is past block {}'s {size} bytes",
                    self.block
                ));
                Err(BadInput("a place past its block (rc2k.log names it)"))
            }
        }
    }

    /// The value here.
    pub fn read<T: Plain>(self, what: &'static str) -> Result<T, BadInput> {
        self.with_bytes(T::LEN, what, |b| T::from_bytes(b))
    }

    /// The value written here.
    pub fn write<T: Plain>(self, value: T, what: &'static str) -> Result<(), BadInput> {
        self.with_bytes(T::LEN, what, |b| value.to_bytes(b))
    }

    /// `len` bytes from here, copied out.
    pub fn read_bytes(self, len: usize, what: &'static str) -> Result<Vec<u8>, BadInput> {
        self.with_bytes(len, what, |b| b.to_vec())
    }

    /// `bytes` written from here on.
    pub fn write_bytes(self, bytes: &[u8], what: &'static str) -> Result<(), BadInput> {
        self.with_bytes(bytes.len(), what, |b| b.copy_from_slice(bytes))
    }

    /// The text from here to its NUL (without it). Refuses one with no NUL in its block.
    pub fn cstring(self, what: &'static str) -> Result<Vec<u8>, BadInput> {
        let block = (self.block as usize)
            .checked_sub(1)
            .and_then(|n| blocks().get(n)?.as_ref().map(|b| Arc::clone(&b.bytes)))
            .ok_or(BadInput("a block that is not there"))?;
        let size = block.lock().unwrap_or_else(|e| e.into_inner()).len();
        let rest = size
            .checked_sub(self.offset as usize)
            .ok_or(BadInput("a text past its block"))?;
        self.with_bytes(rest, what, |b| {
            b.iter().position(|&c| c == 0).map(|end| b[..end].to_vec())
        })?
        .ok_or(BadInput("a text without its end in its block"))
    }
}

/// A variable that names a block (its id, 0 none), laid out as the dword that held the
/// block's address.
#[repr(transparent)]
pub struct BlockVar(crate::session::Dword);

// SAFETY: a `Dword`, a cell of a plain integer.
unsafe impl crate::pointers::Cells for BlockVar {}

impl BlockVar {
    pub const fn new() -> Self {
        Self(crate::session::Dword::new(0))
    }

    /// The block's start, or None.
    pub fn get(&self) -> Option<At> {
        match self.0.get() {
            0 => None,
            id => Some(At::start(id)),
        }
    }

    /// The block's start, or `BadInput(what)` for none.
    pub fn need(&self, what: &'static str) -> Result<At, BadInput> {
        self.get().ok_or(BadInput(what))
    }

    /// The id held (0 none).
    pub fn id(&self) -> u32 {
        self.0.get()
    }

    pub fn set(&self, id: u32) {
        self.0.set(id)
    }

    /// The block whose id a plain dword holds (a copy the game's copiers move as a dword:
    /// the race's `rivals`), or None.
    pub fn of(held: &crate::session::Dword) -> Option<At> {
        match held.get() {
            0 => None,
            id => Some(At::start(id)),
        }
    }
}

impl Default for BlockVar {
    fn default() -> Self {
        Self::new()
    }
}

/// A place that is either in the game's memory (an address: the image's data, until
/// those stores are typed) or in an owned block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Game(u32),
    Owned(At),
}

impl Place {
    /// `n` bytes further on.
    pub fn add(self, n: u32) -> Self {
        match self {
            Place::Game(a) => Place::Game(a.wrapping_add(n)),
            Place::Owned(at) => Place::Owned(at.add(n)),
        }
    }

    pub fn read<T: Plain>(self, what: &'static str) -> Result<T, BadInput> {
        match self {
            Place::Game(a) => crate::pointers::GamePtr::new(a)
                .read_bytes(T::LEN, what)
                .map(|b| T::from_bytes(&b)),
            Place::Owned(at) => at.read(what),
        }
    }

    pub fn write<T: Plain>(self, value: T, what: &'static str) -> Result<(), BadInput> {
        let mut b = vec![0; T::LEN];
        value.to_bytes(&mut b);
        self.write_bytes(&b, what)
    }

    pub fn read_bytes(self, len: usize, what: &'static str) -> Result<Vec<u8>, BadInput> {
        match self {
            Place::Game(a) => crate::pointers::GamePtr::new(a).read_bytes(len, what),
            Place::Owned(at) => at.read_bytes(len, what),
        }
    }

    pub fn write_bytes(self, bytes: &[u8], what: &'static str) -> Result<(), BadInput> {
        match self {
            Place::Game(a) => crate::pointers::GamePtr::new(a).write_bytes(bytes, what),
            Place::Owned(at) => at.write_bytes(bytes, what),
        }
    }

    /// `f` on a copy of the `len` bytes here, written back after (the game's side has no
    /// lent slices).
    pub fn with_bytes<R>(
        self,
        len: usize,
        what: &'static str,
        f: impl FnOnce(&mut [u8]) -> R,
    ) -> Result<R, BadInput> {
        match self {
            Place::Owned(at) => at.with_bytes(len, what, f),
            Place::Game(_) => {
                let mut bytes = self.read_bytes(len, what)?;
                let r = f(&mut bytes);
                self.write_bytes(&bytes, what)?;
                Ok(r)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_are_taken_read_past_their_size_and_go_with_their_list() {
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        // The list in memory lent as a region, as the game's lists are reached.
        let mut memory = vec![0u8; size_of::<MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        let l = memory.as_mut_ptr() as *mut MemoryList;
        let id = take(l, 8).unwrap();
        let at = At::start(id);
        at.add(4).write(0x1234_5678u32, "a test").unwrap();
        assert_eq!(at.add(4).read::<u32>("a test"), Ok(0x1234_5678));
        // Past the 8 bytes asked for: the page's zeros.
        assert_eq!(at.add(0x100).read::<u32>("a test"), Ok(0));
        assert!(at.add(0x1000 - 0x40).read::<u8>("a test").is_err());
        assert_eq!(u32::from_le_bytes(memory[4..8].try_into().unwrap()), 8);
        free_on(l);
        assert!(at.read::<u32>("a test").is_err());
        assert!(At::start(0).read::<u32>("a test").is_err());
        crate::memory::with_figures(|f| *f = figures);
    }
}
