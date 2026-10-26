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
//!
//! Rust's own memory handed on as an address (a static copy of a game literal, a buffer on
//! the stack) is a region too, coarsely: each section of the program's image, and the
//! running thread's stack. These come after the game's blocks and the moved stores, so a
//! read that starts in a store keeps the store's bounds. (Rust buffers become slices later.)

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
    block
        .or_else(|| {
            let (start, end) = crate::moved::home_range(addr as usize)?;
            Some((start as u32, end as u32))
        })
        .or_else(|| {
            own_sections()
                .iter()
                .copied()
                .find(|&(start, end)| (start..end).contains(&addr))
        })
        .or_else(|| {
            let (low, high) = stack();
            (low..high).contains(&addr).then_some((low, high))
        })
}

/// The program's own image sections (statics, constants): first byte and byte past the end.
fn own_sections() -> &'static [(u32, u32)] {
    static SECTIONS: std::sync::OnceLock<Vec<(u32, u32)>> = std::sync::OnceLock::new();
    SECTIONS.get_or_init(|| {
        // SAFETY: the running module's PE headers are mapped and read-only; the offsets are
        // the PE format's (e_lfanew at 0x3C; the file header after the 4-byte signature; the
        // 40-byte section headers after the optional header).
        unsafe {
            use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
            let base = GetModuleHandleW(std::ptr::null()) as usize;
            if base == 0 {
                return Vec::new();
            }
            let read16 = |at: usize| (at as *const u16).read_unaligned() as usize;
            let read32 = |at: usize| (at as *const u32).read_unaligned() as usize;
            let nt = base + read32(base + 0x3C);
            let count = read16(nt + 4 + 2);
            let optional = read16(nt + 4 + 16);
            let first = nt + 4 + 20 + optional;
            (0..count)
                .map(|n| {
                    let header = first + 40 * n;
                    let start = base + read32(header + 12);
                    ((start) as u32, (start + read32(header + 8)) as u32)
                })
                .collect()
        }
    })
}

/// The running thread's stack: its lowest byte and the byte past its top.
fn stack() -> (u32, u32) {
    let (mut low, mut high) = (0usize, 0usize);
    // SAFETY: two out-parameters of the documented call.
    unsafe {
        windows_sys::Win32::System::Threading::GetCurrentThreadStackLimits(&mut low, &mut high)
    };
    (low as u32, high as u32)
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

    /// The `T` written at the address, when all of it lies in one region.
    pub fn write<T: Copy>(self, value: T, what: &str) -> Result<(), BadInput> {
        self.check(size_of::<T>(), what)?;
        // SAFETY: as `read`; the regions are the game's writable memory.
        unsafe { (self.0 as usize as *mut T).write_unaligned(value) };
        Ok(())
    }

    /// `len` bytes from the address, copied out.
    pub fn read_bytes(self, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        self.check(len, what)?;
        // SAFETY: the `len` bytes lie in one live region.
        Ok(unsafe { std::slice::from_raw_parts(self.0 as usize as *const u8, len) }.to_vec())
    }

    /// `bytes` written from the address on.
    pub fn write_bytes(self, bytes: &[u8], what: &str) -> Result<(), BadInput> {
        self.check(bytes.len(), what)?;
        // SAFETY: the bytes lie in one live region; `bytes` is Rust's, apart from it or
        // copied through a temporary by the caller.
        unsafe {
            std::ptr::copy(bytes.as_ptr(), self.0 as usize as *mut u8, bytes.len());
        }
        Ok(())
    }

    /// `len` bytes copied from here to `to` (overlapping as `memmove`), each side inside one
    /// region.
    pub fn copy_to(self, to: GamePtr, len: usize, what: &str) -> Result<(), BadInput> {
        self.check(len, what)?;
        to.check(len, what)?;
        // SAFETY: both runs lie in live regions; `copy` allows overlap.
        unsafe {
            std::ptr::copy(self.0 as usize as *const u8, to.0 as usize as *mut u8, len);
        }
        Ok(())
    }

    /// `len` bytes set to `value` from the address on.
    pub fn fill(self, value: u8, len: usize, what: &str) -> Result<(), BadInput> {
        self.check(len, what)?;
        // SAFETY: the run lies in one live region.
        unsafe { std::ptr::write_bytes(self.0 as usize as *mut u8, value, len) };
        Ok(())
    }

    /// The record of cells at the address (`Cells`: atomics and cells only, so a shared
    /// reference to memory the game also writes is sound), when all of it lies in one region
    /// and the address has the record's alignment. Keep it no longer than the step that
    /// reads it: a block can be given back later.
    pub fn record<T: Cells>(self, what: &str) -> Result<&'static T, BadInput> {
        self.check(size_of::<T>(), what)?;
        if !(self.0 as usize).is_multiple_of(align_of::<T>()) {
            return Err(outside(self.0, size_of::<T>(), what));
        }
        // SAFETY: inside a live region, aligned, and `T` is made of cells (`Cells`).
        Ok(unsafe { &*(self.0 as usize as *const T) })
    }

    /// Whether `len` bytes from the address lie in one region.
    fn check(self, len: usize, what: &str) -> Result<(), BadInput> {
        match region_of(self.0) {
            Some((_, end)) if (end - self.0) as usize >= len => Ok(()),
            _ => Err(outside(self.0, len, what)),
        }
    }
}

/// A type made of atomics and cells only (`session::{Dword, Float, Word, Byte, Bytes<N>}`, their
/// arrays, `#[repr(C)]` records of them), which `GamePtr::record` may hand out shared.
///
/// # Safety
/// Every byte of the type is inside an `UnsafeCell` or an atomic, and any bit pattern is a
/// valid value.
pub unsafe trait Cells {}

// SAFETY: atomics and cells over plain integers.
unsafe impl Cells for crate::session::Dword {}
unsafe impl Cells for crate::session::Float {}
unsafe impl Cells for crate::session::Word {}
unsafe impl Cells for crate::session::Byte {}
unsafe impl<const N: usize> Cells for crate::session::Bytes<N> {}
unsafe impl<T: Cells, const N: usize> Cells for [T; N] {}

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

/// A Rust buffer lent to code that takes the game's addresses (a file read into a `Vec`): a
/// region while the guard lives. Take the guard before the address is handed on.
pub struct Lent(u32);

impl Lent {
    pub fn new(bytes: &[u8]) -> Self {
        let start = bytes.as_ptr() as u32;
        register(start, bytes.len() as u32);
        Self(start)
    }
}

impl Drop for Lent {
    fn drop(&mut self) {
        unregister(self.0);
    }
}

/// The tests' name for a lent buffer.
#[cfg(test)]
pub type TestRegion = Lent;

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::strings::bytes::Bytes;

    #[test]
    fn reads_answer_inside_a_region_only() {
        // On the heap: the stack is a region of its own.
        let block = Box::new([7u8; 64]);
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

    #[test]
    fn rust_statics_and_the_stack_are_regions() {
        static TEXT: [u8; 4] = *b"abc\0";
        let on_stack = [1u8; 8];
        assert!(
            GamePtr::new(TEXT.as_ptr() as u32)
                .read::<[u8; 4]>("test")
                .is_ok()
        );
        assert!(
            GamePtr::new(on_stack.as_ptr() as u32)
                .read::<[u8; 8]>("test")
                .is_ok()
        );
        let heap = Box::new([0u8; 8]);
        assert!(
            GamePtr::new(heap.as_ptr() as u32)
                .read::<u8>("test")
                .is_err()
        );
    }
}
