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

/// A block taken from the system with `VirtualAlloc`: its region is what the system commits,
/// the bytes asked for rounded up to whole pages. The original's reads a few bytes past its
/// block's end land there and always answered (`gfx::rescale_picture`'s last column, up to
/// 3 bytes past its copy; `notes/bugs.md`).
pub fn register_system_block(start: u32, len: u32) {
    const PAGE: u32 = 0x1000;
    register(start, len.div_ceil(PAGE).saturating_mul(PAGE));
}

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
    // Every thread's cached regions are stale now.
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Release);
}

/// Bumped when a block is given back: the threads' caches are then emptied.
static GENERATION: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The exact regions (blocks, moved stores) a thread found last, with the generation they
/// were found in, and the thread's stack.
struct Cache {
    generation: u32,
    regions: [(u32, u32); 4],
    next: usize,
    stack: Option<(u32, u32)>,
}

thread_local! {
    static CACHE: std::cell::RefCell<Cache> = const {
        std::cell::RefCell::new(Cache { generation: u32::MAX, regions: [(0, 0); 4], next: 0, stack: None })
    };
}

/// Whether `len` bytes from `addr` lie in one region, as `region_of` finds them, through the
/// thread's cache: a recent block or store, the stack, then the lookups.
fn inside(addr: u32, len: usize) -> bool {
    let Some(end) = addr.checked_add(len as u32) else {
        return false;
    };
    let fits = |(start, top): (u32, u32)| addr >= start && end <= top;
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let generation = GENERATION.load(std::sync::atomic::Ordering::Acquire);
        if cache.generation != generation {
            cache.generation = generation;
            cache.regions = [(0, 0); 4];
        }
        if cache.regions.iter().any(|&r| fits(r)) {
            return true;
        }
        let stack = *cache.stack.get_or_insert_with(stack);
        if (stack.0..stack.1).contains(&addr) {
            return fits(stack);
        }
        if let Some(region) = exact_region(addr) {
            let next = cache.next;
            cache.regions[next] = region;
            cache.next = (next + 1) % 4;
            return fits(region);
        }
        own_sections()
            .iter()
            .copied()
            .find(|&(s, t)| (s..t).contains(&addr))
            .is_some_and(fits)
    })
}

/// The region `addr` lies in: its first byte and the byte past its end.
pub fn region_of(addr: u32) -> Option<(u32, u32)> {
    exact_region(addr)
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

/// The block or moved store `addr` lies in.
fn exact_region(addr: u32) -> Option<(u32, u32)> {
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

/// The program's own image sections (statics, constants): first byte and byte past the end.
fn own_sections() -> &'static [(u32, u32)] {
    static SECTIONS: std::sync::OnceLock<Vec<(u32, u32)>> = std::sync::OnceLock::new();
    SECTIONS.get_or_init(|| {
        // SAFETY: the running module's PE headers are mapped and read-only; the offsets are
        // the PE format's (e_lfanew at 0x3C; the file header after the 4-byte signature; the
        // 40-byte section headers after the optional header).
        unsafe {
            use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
            let base = host_number(GetModuleHandleW(std::ptr::null()));
            if base == 0 {
                return Vec::new();
            }
            let read16 = |at: usize| (host_at(at) as *const u16).read_unaligned() as usize;
            let read32 = |at: usize| (host_at(at) as *const u32).read_unaligned() as usize;
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

/// Where a game address's bytes are in this process (phase 9, `notes/conversion.md`): the
/// one place a game address becomes a host pointer. On 32 bits the two are the same number;
/// a 64-bit build maps each region's game address to its host base here.
#[inline]
pub fn host(addr: u32) -> *mut u8 {
    std::ptr::with_exposed_provenance_mut(addr as usize)
}

/// A host address held as a number (a moved store's place, `store::game_addr`) as a
/// pointer.
#[inline]
pub fn host_at(addr: usize) -> *mut u8 {
    std::ptr::with_exposed_provenance_mut(addr)
}

/// The game address of a host pointer handed to the game (a lent buffer, a platform
/// object): the inverse of `host`.
#[inline]
pub fn game_address(p: impl HostPointer) -> u32 {
    p.exposed() as u32
}

/// A host pointer as a number (kept as `usize`, a place's address).
#[inline]
pub fn host_number(p: impl HostPointer) -> usize {
    p.exposed()
}

/// A raw pointer of either kind, as `game_address` and `host_number` take it.
pub trait HostPointer {
    fn exposed(self) -> usize;
}

impl<T: ?Sized> HostPointer for *const T {
    fn exposed(self) -> usize {
        self.cast::<u8>().expose_provenance()
    }
}

impl<T: ?Sized> HostPointer for *mut T {
    fn exposed(self) -> usize {
        self.cast::<u8>().expose_provenance()
    }
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
        Ok(unsafe { (host(self.0) as *const T).read_unaligned() })
    }

    /// The `T` written at the address, when all of it lies in one region.
    pub fn write<T: Copy>(self, value: T, what: &str) -> Result<(), BadInput> {
        self.check(size_of::<T>(), what)?;
        // SAFETY: as `read`; the regions are the game's writable memory.
        unsafe { (host(self.0) as *mut T).write_unaligned(value) };
        Ok(())
    }

    /// `len` bytes from the address, copied out.
    pub fn read_bytes(self, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        self.check(len, what)?;
        // SAFETY: the `len` bytes lie in one live region.
        Ok(unsafe { std::slice::from_raw_parts(host(self.0), len) }.to_vec())
    }

    /// `bytes` written from the address on.
    pub fn write_bytes(self, bytes: &[u8], what: &str) -> Result<(), BadInput> {
        self.check(bytes.len(), what)?;
        // SAFETY: the bytes lie in one live region; `bytes` is Rust's, apart from it or
        // copied through a temporary by the caller.
        unsafe {
            std::ptr::copy(bytes.as_ptr(), host(self.0), bytes.len());
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
            std::ptr::copy(host(self.0), host(to.0), len);
        }
        Ok(())
    }

    /// `len` bytes set to `value` from the address on.
    pub fn fill(self, value: u8, len: usize, what: &str) -> Result<(), BadInput> {
        self.check(len, what)?;
        // SAFETY: the run lies in one live region.
        unsafe { std::ptr::write_bytes(host(self.0), value, len) };
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
        Ok(unsafe { &*(host(self.0) as *const T) })
    }

    /// Whether `len` bytes from the address lie in one region.
    fn check(self, len: usize, what: &str) -> Result<(), BadInput> {
        if inside(self.0, len) {
            Ok(())
        } else {
            Err(outside(self.0, len, what))
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
        (at < self.len).then(|| unsafe { host(self.start).add(at).read_volatile() })
    }
}

/// `len` bytes from `addr` checked to lie in one region; outside, the program ends naming
/// them (`fatal::raise::end`). For hot paths (the x87 runs' operands).
pub fn check_or_end(addr: u32, len: usize, what: &str) {
    if !inside(addr, len) {
        crate::fatal::raise::end(what, outside(addr, len, what))
    }
}

/// A checked read where the caller has no way to carry a refusal (generated step code, a
/// core's closure): outside its region the program ends naming it (`fatal::raise::end`).
pub fn read_or_end<T: Copy>(addr: u32, what: &str) -> T {
    GamePtr::new(addr)
        .read(what)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e))
}

/// A checked write, as `read_or_end`.
pub fn write_or_end<T: Copy>(addr: u32, value: T, what: &str) {
    if let Err(e) = GamePtr::new(addr).write(value, what) {
        crate::fatal::raise::end(what, e)
    }
}

/// The last region a run of reads found, kept for the next (a picture read per pixel): a
/// read inside it costs a compare, one outside looks its region up again. Keep a window for
/// one pass only: a block can be given back between passes.
#[derive(Default)]
pub struct Window(std::cell::Cell<(u32, u32)>);

impl Window {
    pub fn new() -> Self {
        Self::default()
    }

    fn check(&self, addr: u32, len: usize, what: &str) -> Result<(), BadInput> {
        let end = addr.checked_add(len as u32);
        let (start, top) = self.0.get();
        if addr >= start && end.is_some_and(|end| end <= top) {
            return Ok(());
        }
        match region_of(addr) {
            Some((start, top)) if end.is_some_and(|end| end <= top) => {
                self.0.set((start, top));
                Ok(())
            }
            _ => Err(outside(addr, len, what)),
        }
    }

    /// A checked read; outside its region the program ends (`fatal::raise::end`).
    pub fn read_or_end<T: Copy>(&self, addr: u32, what: &str) -> T {
        if let Err(e) = self.check(addr, size_of::<T>(), what) {
            crate::fatal::raise::end(what, e)
        }
        // SAFETY: the bytes lie in one live region (checked above).
        unsafe { (host(addr) as *const T).read_unaligned() }
    }

    /// A checked write, as `read_or_end`.
    pub fn write_or_end<T: Copy>(&self, addr: u32, value: T, what: &str) {
        if let Err(e) = self.check(addr, size_of::<T>(), what) {
            crate::fatal::raise::end(what, e)
        }
        // SAFETY: the bytes lie in one live region (checked above).
        unsafe { (host(addr) as *mut T).write_unaligned(value) }
    }

    /// Checked bytes written, as `write_or_end`.
    pub fn write_bytes_or_end(&self, addr: u32, bytes: &[u8], what: &str) {
        if let Err(e) = self.check(addr, bytes.len(), what) {
            crate::fatal::raise::end(what, e)
        }
        // SAFETY: the bytes lie in one live region (checked above).
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), host(addr), bytes.len()) }
    }
}

/// A Rust buffer lent to code that takes the game's addresses (a file read into a `Vec`): a
/// region while the guard lives. Take the guard before the address is handed on.
pub struct Lent(u32);

impl Lent {
    pub fn new(bytes: &[u8]) -> Self {
        let start = game_address(bytes.as_ptr());
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
        let start = game_address(block.as_ptr());
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
    fn a_window_keeps_its_region() {
        let block = Box::new([5u8; 32]);
        let _lent = Lent::new(&block[..]);
        let start = game_address(block.as_ptr());
        let window = Window::new();
        assert_eq!(window.read_or_end::<u8>(start, "test"), 5);
        assert_eq!(window.0.get(), (start, start + 32));
        window.write_or_end(start + 28, 0x0102_0304u32, "test");
        assert_eq!(block[28], 4);
        assert!(
            window.check(start + 29, 4, "test").is_err(),
            "runs past the end"
        );
    }

    #[test]
    fn rust_statics_and_the_stack_are_regions() {
        static TEXT: [u8; 4] = *b"abc\0";
        let on_stack = [1u8; 8];
        assert!(
            GamePtr::new(game_address(TEXT.as_ptr()))
                .read::<[u8; 4]>("test")
                .is_ok()
        );
        assert!(
            GamePtr::new(game_address(on_stack.as_ptr()))
                .read::<[u8; 8]>("test")
                .is_ok()
        );
        let heap = Box::new([0u8; 8]);
        assert!(
            GamePtr::new(game_address(heap.as_ptr()))
                .read::<u8>("test")
                .is_err()
        );
    }
}
