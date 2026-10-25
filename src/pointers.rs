//! The game's own pointers (the 32-bit addresses its data holds), checked against what is
//! there (owner, 2026-10-25: "checked against allocations", `notes/improvements.md`).
//!
//! Every block the game takes from the system (`memory::allocate`, `memory::virtual_alloc`)
//! is registered here with its length and taken off when it is given back
//! (`memory::free_list`); the moved stores (`moved::MOVED`) are regions too. A `GamePtr` is
//! such an address: its reads find the region it lies in and answer only inside it. One
//! outside every region, or a read running past its region's end, is refused: the program
//! ends naming the address (a refusal, `errors::BadInput`), where the original would have
//! read whatever was there.
//!
//! A region is a whole block from the system: the game's lists cut their own pieces out of
//! one (`memory::take_from_list`), and a read past a piece into the next piece of the same
//! block still answers, as the original's did.

use crate::errors::BadInput;
use crate::framework::log;
use std::collections::BTreeMap;
use std::sync::RwLock;

/// The blocks taken from the system: start -> length.
static BLOCKS: RwLock<BTreeMap<u32, u32>> = RwLock::new(BTreeMap::new());

/// A block taken from the system (`len` bytes from `start`, header included).
pub fn register(start: u32, len: u32) {
    if let Ok(mut blocks) = BLOCKS.write() {
        blocks.insert(start, len);
    }
}

/// A block given back.
pub fn unregister(start: u32) {
    if let Ok(mut blocks) = BLOCKS.write() {
        blocks.remove(&start);
    }
}

/// The region `addr` lies in: its first byte and the byte past its end.
pub fn region_of(addr: u32) -> Option<(u32, u32)> {
    let block = BLOCKS.read().ok().and_then(|blocks| {
        let (&start, &len) = blocks.range(..=addr).next_back()?;
        let end = start.checked_add(len)?;
        (addr < end).then_some((start, end))
    });
    block.or_else(|| {
        let (start, end) = crate::moved::home_range(addr as usize)?;
        Some((start as u32, end as u32))
    })
}

/// The refusal for an address that is not where the game's memory is (logged with it).
fn outside(addr: u32, len: usize, what: &str) -> BadInput {
    log(&format!(
        "pointers: {what} at {addr:#010x} ({len} bytes) is outside every allocation"
    ));
    BadInput("a game pointer outside every allocation (rc2k.log names it)")
}

/// An address the game's data holds (a font's glyph table, a list's next record).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GamePtr(u32);

impl GamePtr {
    pub const fn new(addr: u32) -> Self {
        Self(addr)
    }

    pub const fn addr(self) -> u32 {
        self.0
    }

    /// The address `n` bytes on (no check until it is read).
    pub const fn offset(self, n: u32) -> Self {
        Self(self.0.wrapping_add(n))
    }

    /// The bytes from here to the end of the region, for a reader of an extent nobody knows
    /// beforehand (a text up to its NUL, a glyph table): a read past the region's end
    /// answers nothing, which the core refuses.
    pub fn bytes(self, what: &str) -> Result<GameBytes, BadInput> {
        let (_, end) = region_of(self.0).ok_or_else(|| outside(self.0, 1, what))?;
        Ok(GameBytes {
            start: self.0,
            len: (end - self.0) as usize,
        })
    }

    /// The `T` at the address (unaligned, as the game's data may be), when all of it lies in
    /// one region.
    pub fn read<T: Copy>(self, what: &str) -> Result<T, BadInput> {
        self.check(size_of::<T>(), what)?;
        // SAFETY: the whole `T` lies in a live region (a registered block or a moved store).
        Ok(unsafe { (self.0 as usize as *const T).read_unaligned() })
    }

    /// Whether `len` bytes from the address lie in one region.
    fn check(self, len: usize, what: &str) -> Result<(), BadInput> {
        match region_of(self.0) {
            Some((_, end)) if (end - self.0) as usize >= len => Ok(()),
            _ => Err(outside(self.0, len, what)),
        }
    }
}

/// The bytes of a region from a game pointer on (`GamePtr::bytes`).
#[derive(Clone, Copy, Debug)]
pub struct GameBytes {
    start: u32,
    len: usize,
}

impl rc2k_core::strings::bytes::Bytes for GameBytes {
    fn byte(&self, at: usize) -> Option<u8> {
        // SAFETY: `at` is inside the region (checked against its length).
        (at < self.len)
            .then(|| unsafe { (self.start as usize as *const u8).add(at).read_volatile() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::strings::bytes::Bytes;

    #[test]
    fn reads_answer_inside_a_region_only() {
        let block = [7u8; 64];
        let start = block.as_ptr() as u32;
        register(start, 64);
        let p = GamePtr::new(start + 60);
        assert_eq!(p.read::<u32>("test").unwrap(), 0x0707_0707);
        assert!(
            p.offset(1).read::<u32>("test").is_err(),
            "runs past the end"
        );
        let bytes = p.bytes("test").unwrap();
        assert_eq!(bytes.byte(3), Some(7));
        assert_eq!(bytes.byte(4), None);
        unregister(start);
        assert!(p.read::<u8>("test").is_err());
    }

    #[test]
    fn moved_stores_are_regions() {
        let font = crate::gfx::typefaces::FONTS.objects[0].addr() as u32;
        assert!(GamePtr::new(font).read::<[u8; 0x20]>("test").is_ok());
    }
}
