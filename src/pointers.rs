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
//! Rust's own memory is not a region (since 2026-11-05, the memory layer's step 5): its
//! statics are reached as stores or as values, its locals as slices or values, so a game
//! address into the program's sections or a thread's stack is refused, naming its caller.
//! Tests alone keep the coarse regions (each section of the image, the running thread's
//! stack), for the assembly references, which take their buffers by address.

#![allow(unsafe_code)]

use crate::errors::BadInput;
use crate::framework::log;
use std::collections::BTreeMap;
use std::sync::RwLock;

pub mod pool;

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

/// The exact regions (blocks, moved stores) a thread found last, with the generations
/// (blocks given back, game ranges taken off) they were found in, and the thread's stack.
struct Cache {
    generation: u32,
    map_generation: u32,
    regions: [(u32, u32, Backing); 4],
    next: usize,
    /// The region the last check was in.
    last: usize,
    stack: Option<(u32, u32)>,
}

thread_local! {
    static CACHE: std::cell::RefCell<Cache> = const {
        std::cell::RefCell::new(Cache { generation: u32::MAX, map_generation: u32::MAX, regions: [(0, 0, Backing::Raw); 4], next: 0, last: 0, stack: None })
    };
}

/// What holds a region's bytes: a moved range's store (and the game address of its first
/// byte), or plain memory read raw (a block from the system, Rust memory lent by address).
#[derive(Clone, Copy, Default)]
enum Backing {
    Store(&'static dyn Store, u32),
    #[default]
    Raw,
}

impl std::fmt::Debug for Backing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Backing::Store(_, home) => write!(f, "Store({home:#x})"),
            Backing::Raw => f.write_str("Raw"),
        }
    }
}

/// The pool's blocks lent to the game, by their game address: each a store of its own.
static POOL_HOMES: RwLock<BTreeMap<u32, &'static pool::PoolBlock>> = RwLock::new(BTreeMap::new());

/// The backing of the region that starts at `start`.
fn backing_of(start: u32) -> Backing {
    if let Some(&block) = POOL_HOMES.read().ok().as_ref().and_then(|h| h.get(&start)) {
        return Backing::Store(block, start);
    }
    match crate::moved::store_of(start as usize) {
        Some((store, 0)) => Backing::Store(store, start),
        _ => Backing::Raw,
    }
}

/// Whether `len` bytes from `addr` lie in one region, as `region_of` finds them, through the
/// thread's cache: a recent block or store, the stack, then the lookups; and what holds them.
#[inline]
fn inside(addr: u32, len: usize) -> Option<Backing> {
    let end = addr.checked_add(len as u32)?;
    // Most reads follow one another in one region.
    let same_region = CACHE.with(|cache| {
        let cache = cache.borrow();
        let (start, top, backing) = cache.regions[cache.last];
        (addr >= start
            && end <= top
            // Compared only: a change sends the check to the lookups, which take the locks.
            && cache.generation == GENERATION.load(std::sync::atomic::Ordering::Relaxed)
            && cache.map_generation == crate::address_map::generation())
        .then_some(backing)
    });
    same_region.or_else(|| inside_found(addr, end))
}

/// `inside` past its last region: the regions held, the stack, then the lookups.
fn inside_found(addr: u32, end: u32) -> Option<Backing> {
    let fits = |(start, top): (u32, u32)| addr >= start && end <= top;
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let generation = GENERATION.load(std::sync::atomic::Ordering::Acquire);
        let map_generation = crate::address_map::generation();
        if cache.generation != generation || cache.map_generation != map_generation {
            cache.generation = generation;
            cache.map_generation = map_generation;
            cache.regions = [(0, 0, Backing::Raw); 4];
            // A range taken off may have been the stack's (a dead thread's stale range
            // overlapping): it is looked up afresh.
            cache.stack = None;
        }
        if let Some(k) = cache.regions.iter().position(|&(s, t, _)| fits((s, t))) {
            cache.last = k;
            return Some(cache.regions[k].2);
        }
        if cfg!(test) {
            let stack = *cache.stack.get_or_insert_with(|| to_game(stack()));
            if (stack.0..stack.1).contains(&addr) {
                return fits(stack).then_some(Backing::Raw);
            }
        }
        if let Some(region) = exact_region(addr) {
            let next = cache.next;
            let backing = backing_of(region.0);
            cache.regions[next] = (region.0, region.1, backing);
            cache.next = (next + 1) % 4;
            return fits(region).then_some(backing);
        }
        (cfg!(test) && !crate::race::store::is_dead(addr) && own_section(addr).is_some_and(fits))
            .then_some(Backing::Raw)
    })
}

/// The region `addr` lies in: its first byte and the byte past its end.
pub fn region_of(addr: u32) -> Option<(u32, u32)> {
    exact_region(addr)
        .or_else(|| own_section(addr).filter(|_| cfg!(test) && !crate::race::store::is_dead(addr)))
        .or_else(|| {
            if !cfg!(test) {
                return None;
            }
            let (low, high) = to_game(stack());
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
        crate::moved::home_range(addr as usize).map(|(start, end)| (start as u32, end as u32))
    })
}

/// The image section `addr` lies in, as game addresses (the tests' raw regions: the
/// program's places are its stores').
#[cfg(not(test))]
fn own_section(_addr: u32) -> Option<(u32, u32)> {
    None
}

/// The image section `addr` lies in, as game addresses.
#[cfg(test)]
fn own_section(addr: u32) -> Option<(u32, u32)> {
    let at = crate::address_map::host(addr)?;
    own_sections()
        .iter()
        .copied()
        .find(|&(s, t)| (s..t).contains(&at))
        .map(to_game)
}

/// The program's whole image (headers and sections): first byte and byte past the end
/// (host addresses), for `address_map`.
pub(crate) fn own_image() -> (usize, usize) {
    let base = image().0;
    let end = own_sections().iter().map(|&(_, t)| t).max().unwrap_or(base);
    (base, end.next_multiple_of(0x1000))
}

/// The running thread's stack (host addresses), for `address_map`.
pub(crate) fn thread_stack() -> (usize, usize) {
    stack()
}

/// The program's image: base and sections, read once (`os::image`).
fn image() -> &'static (usize, Vec<(usize, usize)>) {
    static IMAGE: std::sync::OnceLock<(usize, Vec<(usize, usize)>)> = std::sync::OnceLock::new();
    IMAGE.get_or_init(crate::os::image)
}

/// The program's own image sections (statics, constants): first byte and byte past the end
/// (host addresses).
fn own_sections() -> &'static [(usize, usize)] {
    &image().1
}

/// The running thread's stack: its lowest byte and the byte past its top (host addresses).
fn stack() -> (usize, usize) {
    crate::os::thread_stack()
}

/// Where a game address's bytes are in this process, through `address_map` (the tests' raw
/// regions only: the program's places are found from their stores, `place_of`). An address no
/// range covers ends the program.
#[cfg(test)]
#[inline]
#[track_caller]
pub fn host(addr: u32) -> *mut u8 {
    if addr == 0 {
        return std::ptr::null_mut();
    }
    if crate::race::course_places::is_place(addr) {
        course_place(addr);
    }
    match crate::address_map::host(addr) {
        Some(at) => std::ptr::with_exposed_provenance_mut(at),
        None => unmapped(addr),
    }
}

/// A course place (`race::course_places`) taken for an address: the reader not converted,
/// named.
#[cfg(test)]
#[cold]
#[track_caller]
fn course_place(addr: u32) -> ! {
    let at = std::panic::Location::caller();
    crate::fatal::raise::end(
        "a course place read as an address",
        BadInput(Box::leak(
            format!("{addr:#x} from {}:{}", at.file(), at.line()).into_boxed_str(),
        )),
    )
}

#[cfg(test)]
#[cold]
fn unmapped(addr: u32) -> ! {
    log(&format!(
        "pointers: the game address {addr:#010x} has no memory behind it"
    ));
    crate::fatal::raise::end(
        "a game address with no memory",
        BadInput("a game address with no memory behind it (rc2k.log names it)"),
    )
}

/// A host range (first byte, byte past the end) as game addresses. The end is the start
/// plus the length: the byte past the end may be the first of another range (memory right
/// above a thread's stack), whose game address is elsewhere.
fn to_game((start, end): (usize, usize)) -> (u32, u32) {
    match game_of(start) {
        0 => (0, 0),
        first => (first, first + (end - start) as u32),
    }
}

/// `host` for a game address held as a `usize` (the tests' buffers and image checks).
#[cfg(test)]
#[inline]
pub fn host_at(addr: usize) -> *mut u8 {
    host(addr as u32)
}

/// The game address of a host pointer handed to the game: the inverse of `host`. The
/// pointer must lie in the program's image, the thread's stack, or memory given a game
/// range (`lend`, `map`); any other ends the program.
#[inline]
pub fn game_address(p: impl HostPointer) -> u32 {
    let at = p.exposed();
    if at == 0 {
        return 0;
    }
    match crate::address_map::game_of(at) {
        Some(game) => game,
        None => {
            log(&format!(
                "pointers: the host address {at:#x} was handed to the game with no game range"
            ));
            crate::fatal::raise::end(
                "a host pointer with no game address",
                BadInput("a host pointer handed to the game with no game range (rc2k.log)"),
            )
        }
    }
}

/// A game range for `len` bytes of host memory at `p` (no region: the memory is reached
/// by address only, as the game's reserved image): its game address.
pub fn map(p: impl HostPointer, len: usize) -> u32 {
    crate::address_map::map(p.exposed(), len)
}

/// Host memory handed to the game until `forget` (the tests' buffers): a game range and a
/// region. Its game address.
#[cfg(test)]
pub fn lend(p: impl HostPointer, len: usize) -> u32 {
    let at = map(p, len);
    register(at, len as u32);
    at
}

/// A block of the pool lent to the game until `forget_block`: a game range over the whole
/// block, a region of `len` bytes from its start (whole pages for a block from the system,
/// `system`: `register_system_block`), the block its store. Its game address.
pub fn lend_block(block: &'static pool::PoolBlock, len: usize, system: bool) -> u32 {
    let at = map(block.as_ptr(), block.len());
    if let Ok(mut homes) = POOL_HOMES.write() {
        homes.insert(at, block);
    }
    if system {
        register_system_block(at, len as u32);
    } else {
        register(at, len as u32);
    }
    at
}

/// The first of `len` bytes at the game address `addr` inside one lent block of the pool (the
/// screen's pixels for the renderers): its host pointer, or None for bytes in no lent block.
pub fn lent_bytes(addr: u32, len: usize) -> Option<*mut u8> {
    let homes = POOL_HOMES.read().ok()?;
    let (&start, block) = homes.range(..=addr).next_back()?;
    let offset = (addr - start) as usize;
    (offset.checked_add(len)? <= block.len()).then(|| block.as_ptr().wrapping_add(offset))
}

/// A lent block given back: its region, its store and its game range; the block, for the
/// pool (None for an address `lend_block` did not hand out).
pub fn forget_block(at: u32) -> Option<&'static pool::PoolBlock> {
    let block = POOL_HOMES.write().ok()?.remove(&at)?;
    forget(at);
    Some(block)
}

/// Lent memory given back: its region and its game range.
pub fn forget(at: u32) {
    unregister(at);
    crate::address_map::unmap(at);
}

/// The game address of a host address held as a number: this module's range tables, and
/// the numbers the system hands over (a fault's addresses, `standalone::on_exception`).
#[inline]
pub fn game_of(addr: usize) -> u32 {
    if addr == 0 {
        return 0;
    }
    crate::address_map::game_of(addr).unwrap_or(0)
}

/// A raw pointer of either kind, as `game_address` takes it.
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
/// Ranges of game memory watched while a store moves out of them (phase 13): each access
/// through the checked paths (`GamePtr`, `check_or_end`, `read_or_end`, `Window`) that
/// touches one is logged once per source line of its first caller outside this file (with
/// `#[track_caller]` up to there) and then goes on as before, so one run lists every
/// reader by address. Up to four: (start, end, name).
static WATCHED: [std::sync::atomic::AtomicU64; 4] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 4];
static WATCH_NAMES: std::sync::Mutex<[&'static str; 4]> = std::sync::Mutex::new([""; 4]);
/// The (file, line, range) already logged.
type Seen = std::collections::HashSet<(&'static str, u32, &'static str)>;
static SEEN: std::sync::Mutex<Option<Seen>> = std::sync::Mutex::new(None);

/// `start .. end` watched under `name` (replacing a range of the same name; an empty range
/// ends the watch). At most four at once: a fifth is not watched.
pub fn watch(name: &'static str, start: u32, end: u32) {
    use std::sync::atomic::Ordering::Release;
    let mut names = WATCH_NAMES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let slot = names
        .iter()
        .position(|n| *n == name)
        .or_else(|| names.iter().position(|n| n.is_empty()));
    let Some(k) = slot else { return };
    if start < end {
        names[k] = name;
        ANY_WATCHED.store(true, std::sync::atomic::Ordering::Relaxed);
        WATCHED[k].store(start as u64 | (end as u64) << 32, Release);
    } else {
        names[k] = "";
        WATCHED[k].store(0, Release);
    }
}

/// The watched range `name` no longer watched.
pub fn unwatch(name: &'static str) {
    watch(name, 0, 0);
}

/// Whether any range has been watched: the check of every by-address access, kept cheap.
static ANY_WATCHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[track_caller]
#[inline]
fn watched(addr: u32, len: usize, what: &str) {
    if ANY_WATCHED.load(std::sync::atomic::Ordering::Relaxed) {
        watched_slowly(addr, len, what);
    }
}

#[track_caller]
fn watched_slowly(addr: u32, len: usize, what: &str) {
    let end = addr.saturating_add(len as u32);
    for (k, w) in WATCHED.iter().enumerate() {
        let w = w.load(std::sync::atomic::Ordering::Relaxed);
        let (start, top) = (w as u32, (w >> 32) as u32);
        if w == 0 || addr >= top || end <= start {
            continue;
        }
        let name = WATCH_NAMES.lock().map(|n| n[k]).unwrap_or("");
        let at = std::panic::Location::caller();
        let first = SEEN
            .lock()
            .map(|mut s| {
                s.get_or_insert_default()
                    .insert((at.file(), at.line(), name))
            })
            .unwrap_or(false);
        if first {
            log(&format!(
                "watch: {name} +{:#x} ({len} bytes, {what}) by address from {}:{}",
                addr - start,
                at.file(),
                at.line()
            ));
        }
        return;
    }
}

#[track_caller]
fn outside(addr: u32, len: usize, what: &str) -> BadInput {
    let at = std::panic::Location::caller();
    let place = if crate::race::course_places::is_place(addr) {
        " (a course place read as an address)"
    } else {
        ""
    };
    log(&format!(
        "pointers: {what} at {addr:#010x} ({len} bytes) is outside every allocation{place}, from {}:{}",
        at.file(),
        at.line()
    ));
    BadInput("a game pointer outside every allocation (rc2k.log names it)")
}

/// An address the game's data holds (a font's glyph table, a list's next record).
///
/// One made from a field (`to`) also keeps that field's ends: going outside them is refused, so an
/// overrun inside a store is seen, which a region's check is not (a chat message over the
/// network's mode, 2026-11-01). `unbounded` marks the places the original reads on.
#[derive(Clone, Copy, Debug)]
pub struct GamePtr {
    at: u32,
    /// The field's first byte and the byte past it; both 0 for no field.
    field: (u32, u32),
}

impl PartialEq for GamePtr {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}

impl Eq for GamePtr {}

impl GamePtr {
    pub const fn new(addr: u32) -> Self {
        Self {
            at: addr,
            field: (0, 0),
        }
    }

    /// The same address, free of its field's ends: where the original runs on past a field.
    pub const fn unbounded(self) -> Self {
        Self::new(self.at)
    }

    /// The game address of a field of Rust's game state (a store's cell, buffer or place).
    pub fn to<T: ?Sized>(field: &T) -> Self {
        let at = game_address(std::ptr::from_ref::<T>(field).cast::<u8>());
        Self {
            at,
            field: (at, at.wrapping_add(size_of_val(field) as u32)),
        }
    }

    pub const fn is_null(self) -> bool {
        self.at == 0
    }

    pub const fn addr(self) -> u32 {
        self.at
    }

    /// The same address, limited to `len` bytes from here: a buffer that is part of a field.
    pub const fn limited(self, len: u32) -> Self {
        Self {
            at: self.at,
            field: (self.at, self.at.wrapping_add(len)),
        }
    }

    /// The address `n` bytes on (no check until it is read).
    pub const fn offset(self, n: u32) -> Self {
        Self {
            at: self.at.wrapping_add(n),
            field: self.field,
        }
    }

    /// The bytes from here to the end of the region, for a reader of an extent nobody knows
    /// beforehand (a text up to its NUL, a glyph table): a read past the region's end
    /// answers nothing, which the core refuses.
    #[track_caller]
    pub fn bytes(self, what: &str) -> Result<GameBytes, BadInput> {
        watched(self.at, 1, what);
        let (first, end) = region_of(self.at).ok_or_else(|| outside(self.at, 1, what))?;
        // Inside its field, when it has one.
        let end = if self.field.1 == 0 {
            end
        } else {
            end.min(self.field.1)
        };
        Ok(GameBytes {
            start: self.at,
            len: end.saturating_sub(self.at) as usize,
            backing: backing_of(first),
        })
    }

    /// `len` bytes from the address as a slice, when all of them lie in one region; else the
    /// program ends naming the address.
    ///
    /// # Safety
    /// Nothing may write the bytes while the slice lives (the game's memory is shared).
    #[track_caller]
    pub unsafe fn slice<'a>(self, len: usize, what: &str) -> &'a [u8] {
        let backing = self
            .check(len, what)
            .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
        // SAFETY: the bytes lie in one live region (checked above); unwritten meanwhile
        // (the caller's promise).
        unsafe { std::slice::from_raw_parts(place_of(self.at, backing), len) }
    }

    /// The `T` at the address (unaligned, as the game's data may be), when all of it lies in
    /// one region.
    #[track_caller]
    pub fn read<T: Plain>(self, what: &str) -> Result<T, BadInput> {
        let backing = self.check(size_of::<T>(), what)?;
        let mut value = zeroed::<T>();
        load_at(self.at, backing, bytes_mut(&mut value))?;
        Ok(value)
    }

    /// The `T` written at the address, when all of it lies in one region.
    #[track_caller]
    pub fn write<T: Plain>(self, value: T, what: &str) -> Result<(), BadInput> {
        let backing = self.check(size_of::<T>(), what)?;
        store_at(self.at, backing, bytes_of(&value))
    }

    /// `len` bytes from the address, copied out.
    #[track_caller]
    pub fn read_bytes(self, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        let backing = self.check(len, what)?;
        let mut bytes = vec![0; len];
        load_at(self.at, backing, &mut bytes)?;
        Ok(bytes)
    }

    /// `bytes` written from the address on.
    #[track_caller]
    pub fn write_bytes(self, bytes: &[u8], what: &str) -> Result<(), BadInput> {
        let backing = self.check(bytes.len(), what)?;
        store_at(self.at, backing, bytes)
    }

    /// `len` bytes copied from here to `to` (overlapping as `memmove`), each side inside one
    /// region.
    #[track_caller]
    pub fn copy_to(self, to: GamePtr, len: usize, what: &str) -> Result<(), BadInput> {
        let (from, into) = (self.check(len, what)?, to.check(len, what)?);
        if matches!(from, Backing::Store(..)) || matches!(into, Backing::Store(..)) {
            // Through a copy, as `memmove`: either side may be a store's cells.
            let mut bytes = vec![0; len];
            load_at(self.at, from, &mut bytes)?;
            return store_at(to.at, into, &bytes);
        }
        // SAFETY: both runs lie in live regions; `copy` allows overlap.
        unsafe {
            crate::shared_memory::copy(raw_place(self.at), raw_place(to.at), len);
        }
        Ok(())
    }

    /// `len` bytes set to `value` from the address on.
    #[track_caller]
    pub fn fill(self, value: u8, len: usize, what: &str) -> Result<(), BadInput> {
        if let backing @ Backing::Store(..) = self.check(len, what)? {
            return store_at(self.at, backing, &vec![value; len]);
        }
        // SAFETY: the run lies in one live region.
        unsafe { crate::shared_memory::fill(raw_place(self.at), value, len) };
        Ok(())
    }

    /// The record of cells at the address (`Cells`: atomics and cells only, so a shared
    /// reference to memory the game also writes is sound), when all of it lies in one region
    /// and the address has the record's alignment. Keep it no longer than the step that
    /// reads it: a block can be given back later.
    #[track_caller]
    pub fn record<T: Cells>(self, what: &str) -> Result<&'static T, BadInput> {
        let backing = self.check(size_of::<T>(), what)?;
        let place = place_of(self.at, backing).cast::<T>();
        if !place.is_aligned() {
            return Err(outside(self.at, size_of::<T>(), what));
        }
        // SAFETY: inside a live region, aligned, and `T` is made of cells (`Cells`).
        Ok(unsafe { &*place })
    }

    /// Whether `len` bytes from the address lie in one region.
    #[track_caller]
    fn check(self, len: usize, what: &str) -> Result<Backing, BadInput> {
        watched(self.at, len, what);
        let (first, end) = self.field;
        if end != 0 && (self.at < first || u64::from(self.at) + len as u64 > u64::from(end)) {
            log(&format!(
                "pointers: {what}: {len} bytes at {:#x}, outside its field {first:#x} .. {end:#x}",
                self.at
            ));
            return Err(BadInput("past the end of a field"));
        }
        inside(self.at, len).ok_or_else(|| outside(self.at, len, what))
    }
}

/// A type made of atomics and cells only (`session::{Dword, Float, Word, Byte, Bytes<N>}`, their
/// arrays, `#[repr(C)]` records of them), which `GamePtr::record` may hand out shared.
///
/// # Safety
/// Every byte of the type is inside an `UnsafeCell` or an atomic, and any bit pattern is a
/// valid value. `cell_at` names, for each byte below the type's size, the primitive cell that
/// holds it: an atomic of that width at that offset, or a run of `Bytes`.
pub unsafe trait Cells {
    /// The primitive cell holding byte `offset` (below the type's size), placed from the
    /// record's first byte.
    fn cell_at(offset: usize) -> CellAt;

    /// The record's runs of like cells in order (each the first cell of its run), when the
    /// type keeps them (the derive does for records without generics, built at first use
    /// by `flatten`): a lookup is then one search, and a copy walks them in turn.
    fn runs() -> Option<&'static [CellAt]> {
        None
    }
}

/// `T`'s runs of like cells, in order (`Cells::runs`).
pub fn flatten<T: Cells>() -> Vec<CellAt> {
    let mut runs = Vec::new();
    let mut at = 0;
    while at < size_of::<T>() {
        let cell = T::cell_at(at);
        runs.push(cell);
        at = cell.run_end;
    }
    runs
}

/// The cell holding byte `offset` of a record whose runs are `runs`, and the run's index.
fn in_runs(runs: &[CellAt], offset: usize) -> (usize, CellAt) {
    let i = runs.partition_point(|run| run.start <= offset) - 1;
    let run = runs[i];
    let start = run.start + (offset - run.start) / run.len * run.len;
    (i, CellAt { start, ..run })
}

/// The cell holding byte `offset` of a `T`: through its runs when it keeps them.
fn cell_of<T: Cells>(offset: usize) -> CellAt {
    match T::runs() {
        Some(runs) => in_runs(runs, offset).1,
        None => T::cell_at(offset),
    }
}

/// What a primitive cell is: an atomic of one width, or a run of plain shared bytes
/// (`Bytes<N>`, copied as `shared_memory` copies).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKind {
    U8,
    U16,
    U32,
    Bytes,
}

/// A primitive cell of a record: where it starts, how long it is, what it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellAt {
    pub start: usize,
    pub len: usize,
    pub kind: CellKind,
    /// The end of the run of cells like this one that it begins or lies in (an array of
    /// one primitive cell: the array's end), so a copy steps through the run without asking
    /// again.
    pub run_end: usize,
}

impl CellAt {
    const fn atomic(kind: CellKind, len: usize) -> Self {
        Self {
            start: 0,
            len,
            kind,
            run_end: len,
        }
    }

    /// The same cell in a record where its holder starts `at` bytes in.
    pub const fn after(self, at: usize) -> Self {
        Self {
            start: self.start + at,
            run_end: self.run_end + at,
            ..self
        }
    }
}

// SAFETY: `#[repr(C)]` over its one array of `Dword`s.
unsafe impl<const N: usize> Cells for crate::menu::draw_lists_store::DrawList<N> {
    fn cell_at(offset: usize) -> CellAt {
        <[crate::session::Dword; N]>::cell_at(offset)
    }
}
// SAFETY: one `AtomicU32`.
unsafe impl Cells for crate::session::Dword {
    fn cell_at(_: usize) -> CellAt {
        CellAt::atomic(CellKind::U32, 4)
    }
}
// SAFETY: one `AtomicU32`.
unsafe impl Cells for crate::session::Float {
    fn cell_at(_: usize) -> CellAt {
        CellAt::atomic(CellKind::U32, 4)
    }
}
// SAFETY: one `AtomicU16`.
unsafe impl Cells for crate::session::Word {
    fn cell_at(_: usize) -> CellAt {
        CellAt::atomic(CellKind::U16, 2)
    }
}
// SAFETY: one `AtomicU8`.
unsafe impl Cells for crate::session::Byte {
    fn cell_at(_: usize) -> CellAt {
        CellAt::atomic(CellKind::U8, 1)
    }
}
// SAFETY: an `UnsafeCell` over plain bytes, all of it one run.
unsafe impl<const N: usize> Cells for crate::session::Bytes<N> {
    fn cell_at(_: usize) -> CellAt {
        CellAt::atomic(CellKind::Bytes, N)
    }
}
// SAFETY: an `UnsafeCell` over a plain record (any bytes make one, `Plain`), at the widths
// its `get` / `set` take (`Shared::WORDS`): a 4-aligned record's whole dwords one run of
// `AtomicU32`s, the bytes past them `AtomicU8`s; a packed record one run of shared bytes.
unsafe impl<T: Plain> Cells for crate::session::Shared<T> {
    fn cell_at(offset: usize) -> CellAt {
        let size = size_of::<T>();
        let words = size & !3;
        if !crate::session::Shared::<T>::WORDS {
            CellAt::atomic(CellKind::Bytes, size)
        } else if offset < words {
            CellAt {
                start: offset & !3,
                len: 4,
                kind: CellKind::U32,
                run_end: words,
            }
        } else {
            CellAt {
                start: offset,
                len: 1,
                kind: CellKind::U8,
                run_end: size,
            }
        }
    }
}
// SAFETY: an `UnsafeCell` over four plain bytes, all of it one run.
unsafe impl Cells for crate::session::Loose {
    fn cell_at(_: usize) -> CellAt {
        CellAt::atomic(CellKind::Bytes, 4)
    }
}
// SAFETY: `N` of a type of cells, one after another.
unsafe impl<T: Cells, const N: usize> Cells for [T; N] {
    fn cell_at(offset: usize) -> CellAt {
        let size = size_of::<T>();
        assert!(
            size != 0 && offset < size * N,
            "Cells: byte {offset} outside the array"
        );
        let element = offset - offset % size;
        let inner = T::cell_at(offset % size);
        if inner.start == 0 && inner.len == size {
            // An array of one primitive cell: one run to the array's end.
            CellAt {
                start: element,
                run_end: size * N,
                ..inner
            }
        } else {
            inner.after(element)
        }
    }
}

/// A value any bytes make (integers, floats, arrays of them): what a game place's bytes are
/// read as.
///
/// # Safety
/// Every bit pattern of `size_of::<Self>()` bytes is a valid value, and it has no padding.
pub unsafe trait Plain: Copy {}

macro_rules! plain {
    ($($t:ty),*) => { $(
        // SAFETY: an integer or float: any bits are a value, no padding.
        unsafe impl Plain for $t {}
    )* };
}
plain!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
// SAFETY: `N` plain values one after another.
unsafe impl<T: Plain, const N: usize> Plain for [T; N] {}

/// A `T` of zero bytes.
fn zeroed<T: Plain>() -> T {
    // SAFETY: any bytes make a `T` (`Plain`), zeros among them.
    unsafe { std::mem::zeroed() }
}

/// The bytes of a `T`, to be read into.
fn bytes_mut<T: Plain>(value: &mut T) -> &mut [u8] {
    // SAFETY: a `T` has no padding and any bytes make one (`Plain`); the slice borrows it.
    unsafe { std::slice::from_raw_parts_mut(std::ptr::from_mut(value).cast(), size_of::<T>()) }
}

/// The bytes of a `T`.
fn bytes_of<T: Plain>(value: &T) -> &[u8] {
    // SAFETY: a `T` has no padding (`Plain`); the slice borrows it.
    unsafe { std::slice::from_raw_parts(std::ptr::from_ref(value).cast(), size_of::<T>()) }
}

/// `to.len()` bytes from the game address `addr`, which the caller checked to lie in one
/// region (`backing`, what holds it): through its store when a moved range holds it, else the region's raw bytes (the
/// one raw read left: blocks from the system, Rust memory lent by address).
fn load_at(addr: u32, backing: Backing, to: &mut [u8]) -> Result<(), BadInput> {
    if let Backing::Store(store, home) = backing {
        let at = (addr - home) as usize;
        match run_of(store, at, to.len()) {
            Some(Run::Cell(place, kind)) => {
                to.copy_from_slice(&cell_bytes(place, kind)[..to.len()]);
                return Ok(());
            }
            Some(Run::Bytes(place)) => {
                // SAFETY: inside one run of the store's shared bytes (`run_of`).
                unsafe { crate::shared_memory::load(place, to.as_mut_ptr(), to.len()) };
                return Ok(());
            }
            None => return store.load(at, to),
        }
    }
    // SAFETY: the run lies in one live region (the caller's check); `to` is Rust's.
    unsafe { crate::shared_memory::load(raw_place(addr), to.as_mut_ptr(), to.len()) };
    Ok(())
}

/// `from` written at the game address `addr`, as `load_at` reads.
fn store_at(addr: u32, backing: Backing, from: &[u8]) -> Result<(), BadInput> {
    if let Backing::Store(store, home) = backing {
        let at = (addr - home) as usize;
        match run_of(store, at, from.len()) {
            Some(Run::Cell(place, kind)) => {
                set_cell_bytes(place, kind, 0, from);
                return Ok(());
            }
            Some(Run::Bytes(place)) => {
                // SAFETY: as `load_at`'s.
                unsafe { crate::shared_memory::store(place.cast_mut(), from.as_ptr(), from.len()) };
                return Ok(());
            }
            None => return store.store(at, from),
        }
    }
    // SAFETY: as `load_at`'s; the regions are the game's writable memory.
    unsafe { crate::shared_memory::store(raw_place(addr), from.as_ptr(), from.len()) };
    Ok(())
}

/// Where `addr`'s bytes are in the region `backing` holds (checked by the caller): in its
/// store's bytes, from the store's first one.
fn place_of(addr: u32, backing: Backing) -> *mut u8 {
    match backing {
        Backing::Store(store, home) => store
            .first()
            .cast_mut()
            .wrapping_add((addr - home) as usize),
        Backing::Raw => raw_place(addr),
    }
}

/// A raw region's bytes: the tests' only (their stack, the image's sections, buffers lent by
/// address); every region of the program is a store's.
#[cfg(test)]
fn raw_place(addr: u32) -> *mut u8 {
    host(addr)
}

#[cfg(not(test))]
#[cold]
#[track_caller]
fn raw_place(addr: u32) -> *mut u8 {
    crate::fatal::raise::end(
        "a raw region",
        BadInput(Box::leak(
            format!("{addr:#x} in a region no store holds").into_boxed_str(),
        )),
    )
}

/// Where a short access in a store goes straight: one whole atomic cell of the access's width,
/// or a part of a run of shared bytes.
enum Run {
    Cell(*const u8, CellKind),
    Bytes(*const u8),
}

/// The fast way to `len` bytes at `at` in `store`, when there is one: through the thread's
/// last run of like cells (most accesses step through one table), else its cell looked up.
fn run_of(store: &'static dyn Store, at: usize, len: usize) -> Option<Run> {
    thread_local! {
        static LAST: std::cell::Cell<Option<(*const u8, CellAt)>> =
            const { std::cell::Cell::new(None) };
    }
    let first = store.first();
    let cell = match LAST.get() {
        Some((was, cell)) if was == first && (cell.start..cell.run_end).contains(&at) => cell,
        _ => {
            let cell = store.cell(at)?;
            LAST.set(Some((first, cell)));
            cell
        }
    };
    if at + len > cell.run_end {
        return None;
    }
    let place = first.wrapping_add(at);
    match cell.kind {
        CellKind::Bytes => Some(Run::Bytes(place)),
        // One whole cell of the run, of the access's width.
        kind => ((at - cell.start).is_multiple_of(cell.len) && len == cell.len)
            .then_some(Run::Cell(place, kind)),
    }
}

/// `#[repr(C)]` records of integers, their sizes the sums of their fields (no padding).
macro_rules! plain_record {
    ($($t:ty = $size:expr),* $(,)?) => { $(
        const _: () = assert!(size_of::<$t>() == $size, "padding in a plain record");
        // SAFETY: `#[repr(C)]` over integers only, no padding (asserted above).
        unsafe impl Plain for $t {}
    )* };
}
plain_record!(
    rc2k_core::text::Font = 4 + 7 * 4,
    rc2k_core::memory::List = 4 * 4,
    rc2k_core::gfx::pixels::PixelFormat = 5 * 4,
    crate::gfx::texture_upload::TextureFormat = 3 * 4 + 5 * 4 + 8 * 4,
    crate::gfx::texture_upload::TextureRecord = (5 + 9 + 9 + 7) * 4,
    crate::audio::samples::SampleFile = 0x20,
    rc2k_core::audio::streams::AudioStream = 0x2C,
    crate::audio::make_buffer::BufferDescription = 0x14,
    crate::audio::describe_wave_format::WaveFormat = 0x12,
    rc2k_core::audio::modes::AudioOptions = 0x40,
    rc2k_core::audio::voices::SampleRecord = 0x40,
    rc2k_core::audio::voices::Voice = 0x70,
    rc2k_core::audio::voices::MixerChannel = 0x58,
    rc2k_core::screens::car_pages::CarDescription = 0x78,
    rc2k_core::menu::player::Player = 0x28C,
);

/// One of the game's stores: a static whose every byte is a cell, read and written by
/// offset through its own cells (`load_cells`, `store_cells`). What a game address in a moved
/// range resolves to (`moved::Home`).
///
/// # Safety
/// `cell(offset)`, when it answers, names the primitive cell at `first() + offset` truly (an
/// atomic of that width there, or a run of shared bytes), as `Cells::cell_at` does: reads and
/// writes go straight to it.
pub unsafe trait Store: Sync {
    /// Its length in bytes.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "GamePtr reads through stores at step 4 (notes/memory_layer_plan.md)"
        )
    )]
    fn size(&self) -> usize;
    /// Its first byte: a checked place's bytes are found from it (`place_of`), inside `size()`.
    fn first(&self) -> *const u8;
    /// The primitive cell around byte `offset`, placed from `first()`; `None` where it begins
    /// before `first()` (the reads there take `load` and `store`).
    fn cell(&self, offset: usize) -> Option<CellAt>;
    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), BadInput>;
    fn store(&self, offset: usize, from: &[u8]) -> Result<(), BadInput>;
}

// SAFETY: `cell` is `Cells::cell_at`, from the record's first byte.
unsafe impl<T: Cells + Sync> Store for T {
    fn size(&self) -> usize {
        size_of::<T>()
    }

    fn cell(&self, offset: usize) -> Option<CellAt> {
        Some(cell_of::<T>(offset))
    }

    fn first(&self) -> *const u8 {
        std::ptr::from_ref(self).cast()
    }

    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), BadInput> {
        load_cells(self, offset, to)
    }

    fn store(&self, offset: usize, from: &[u8]) -> Result<(), BadInput> {
        store_cells(self, offset, from)
    }
}

/// The bytes of a cell's value, as many as it is wide.
fn cell_bytes(place: *const u8, kind: CellKind) -> [u8; 4] {
    use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, Ordering::Relaxed};
    let mut bytes = [0; 4];
    // SAFETY: `place` is an atomic of this width inside a borrowed record (`Cells::cell_at`).
    unsafe {
        match kind {
            CellKind::U8 => bytes[0] = AtomicU8::from_ptr(place.cast_mut()).load(Relaxed),
            CellKind::U16 => bytes[..2].copy_from_slice(
                &AtomicU16::from_ptr(place.cast_mut().cast())
                    .load(Relaxed)
                    .to_ne_bytes(),
            ),
            CellKind::U32 => {
                bytes = AtomicU32::from_ptr(place.cast_mut().cast())
                    .load(Relaxed)
                    .to_ne_bytes();
            }
            CellKind::Bytes => unreachable!("a run of bytes is no atomic"),
        }
    }
    bytes
}

/// A cell's bytes `from ..` set to `new`, the others kept: one store when all of it is new,
/// else one compare-and-swap, so a byte written into a dword never undoes a write to the
/// dword's other bytes.
fn set_cell_bytes(place: *const u8, kind: CellKind, from: usize, new: &[u8]) {
    use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, Ordering::Relaxed};
    let merge = |old: [u8; 4]| {
        let mut bytes = old;
        bytes[from..from + new.len()].copy_from_slice(new);
        bytes
    };
    // SAFETY: as `cell_bytes`'.
    unsafe {
        match kind {
            CellKind::U8 => AtomicU8::from_ptr(place.cast_mut()).store(new[0], Relaxed),
            CellKind::U16 => {
                let cell = AtomicU16::from_ptr(place.cast_mut().cast());
                let _ = cell.try_update(Relaxed, Relaxed, |old| {
                    let [a, b, ..] = old.to_ne_bytes();
                    let [a, b, ..] = merge([a, b, 0, 0]);
                    Some(u16::from_ne_bytes([a, b]))
                });
            }
            CellKind::U32 => {
                let cell = AtomicU32::from_ptr(place.cast_mut().cast());
                if new.len() == 4 {
                    cell.store(u32::from_ne_bytes(merge([0; 4])), Relaxed);
                } else {
                    let _ = cell.try_update(Relaxed, Relaxed, |old| {
                        Some(u32::from_ne_bytes(merge(old.to_ne_bytes())))
                    });
                }
            }
            CellKind::Bytes => unreachable!("a run of bytes is no atomic"),
        }
    }
}

/// The cells `offset .. offset + len` of `record` that `visit(cell, from, to)` gets in turn:
/// the cell, and the part of it in the run (offsets from the record's first byte).
fn each_cell<T: Cells>(
    offset: usize,
    len: usize,
    mut visit: impl FnMut(CellAt, usize, usize),
) -> Result<(), BadInput> {
    let end = offset
        .checked_add(len)
        .filter(|&end| end <= size_of::<T>())
        .ok_or(BadInput("past the end of a record"))?;
    let runs = T::runs();
    let mut next: Option<usize> = None;
    let mut at = offset;
    while at < end {
        // The cell at `at` (the next run, when the record keeps its runs), then the rest of
        // its run, cell by cell.
        let mut cell = match (runs, next) {
            (Some(runs), Some(i)) if i < runs.len() && runs[i].start == at => {
                next = Some(i + 1);
                runs[i]
            }
            (Some(runs), _) => {
                let (i, cell) = in_runs(runs, at);
                next = Some(i + 1);
                cell
            }
            (None, _) => T::cell_at(at),
        };
        let run_end = end.min(cell.run_end);
        loop {
            let to = run_end.min(cell.start + cell.len);
            visit(cell, at, to);
            at = to;
            if at >= run_end {
                break;
            }
            cell.start += cell.len;
        }
    }
    Ok(())
}

/// `to.len()` bytes of `record` from byte `offset` on, each cell read whole at its own
/// width (a run of `Bytes` as `shared_memory` reads it).
pub fn load_cells<T: Cells>(record: &T, offset: usize, to: &mut [u8]) -> Result<(), BadInput> {
    let base = std::ptr::from_ref(record).cast::<u8>();
    each_cell::<T>(offset, to.len(), |cell, from, upto| {
        let out = &mut to[from - offset..upto - offset];
        if cell.kind == CellKind::Bytes {
            // SAFETY: the run lies inside the record's `Bytes` (`cell_at`), shared bytes
            // copied through `shared_memory`; `out` is the caller's.
            unsafe { crate::shared_memory::load(base.add(from), out.as_mut_ptr(), out.len()) };
        } else {
            // SAFETY: the cell lies inside the record (`cell_at`).
            let bytes = cell_bytes(unsafe { base.add(cell.start) }, cell.kind);
            out.copy_from_slice(&bytes[from - cell.start..upto - cell.start]);
        }
    })
}

/// `from` written into `record` from byte `offset` on, each cell at its own width (`set_cell_bytes`).
pub fn store_cells<T: Cells>(record: &T, offset: usize, from: &[u8]) -> Result<(), BadInput> {
    let base = std::ptr::from_ref(record).cast::<u8>();
    each_cell::<T>(offset, from.len(), |cell, at, upto| {
        let new = &from[at - offset..upto - offset];
        if cell.kind == CellKind::Bytes {
            // SAFETY: as `load_cells`'.
            unsafe {
                crate::shared_memory::store(base.add(at).cast_mut(), new.as_ptr(), new.len());
            }
        } else {
            // SAFETY: the cell lies inside the record (`cell_at`).
            set_cell_bytes(
                unsafe { base.add(cell.start) },
                cell.kind,
                at - cell.start,
                new,
            );
        }
    })
}

/// A record of cells with these bytes as its first values (a static's, at compile time:
/// a race-block piece that left the block takes the image's bytes).
pub const fn cells_of_bytes<T: Cells, const N: usize>(bytes: [u8; N]) -> T {
    assert!(core::mem::size_of::<T>() == N, "the record's size");
    union Bits<T, const N: usize> {
        bytes: core::mem::ManuallyDrop<[u8; N]>,
        record: core::mem::ManuallyDrop<T>,
    }
    let bits = Bits::<T, N> {
        bytes: core::mem::ManuallyDrop::new(bytes),
    };
    // SAFETY: `T` is made of cells over plain integers (`Cells`), so any bytes are a valid
    // value, and the sizes are equal (checked above).
    core::mem::ManuallyDrop::into_inner(unsafe { bits.record })
}

/// `N` dwords of a record of cells from byte `at` on, for a record read as a run of dwords.
pub fn dwords_in<T: Cells, const N: usize>(cells: &T, at: usize) -> &[crate::session::Dword; N] {
    use crate::session::Dword;
    assert!(
        at + N * size_of::<Dword>() <= size_of::<T>(),
        "dwords past the record"
    );
    let first = std::ptr::from_ref::<T>(cells).cast::<u8>().wrapping_add(at);
    assert!(first.cast::<Dword>().is_aligned(), "dwords on an odd place");
    // SAFETY: inside `cells`, whose every byte is a cell (`Cells`) for as long as the borrow,
    // aligned (checked above), and any bits make a `Dword`.
    unsafe { &*first.cast() }
}

/// The bytes of a region from a game pointer on (`GamePtr::bytes`).
#[derive(Clone, Copy, Debug)]
pub struct GameBytes {
    start: u32,
    len: usize,
    backing: Backing,
}

impl rc2k_core::strings::bytes::Bytes for GameBytes {
    fn byte(&self, at: usize) -> Option<u8> {
        let mut byte = [0];
        // `at` is inside the region (checked against its length).
        (at < self.len && load_at(self.start + at as u32, self.backing, &mut byte).is_ok())
            .then_some(byte[0])
    }
}

/// `len` bytes from `addr` checked to lie in one region; outside, the program ends naming
/// them (`fatal::raise::end`). For hot paths (the x87 runs' operands).
#[track_caller]
pub fn check_or_end(addr: u32, len: usize, what: &str) {
    watched(addr, len, what);
    if inside(addr, len).is_none() {
        crate::fatal::raise::end(what, outside(addr, len, what))
    }
}

/// A checked read where the caller has no way to carry a refusal (generated step code, a
/// core's closure): outside its region the program ends naming it (`fatal::raise::end`).
#[track_caller]
pub fn read_or_end<T: Plain>(addr: u32, what: &str) -> T {
    GamePtr::new(addr)
        .read(what)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e))
}

/// The dword cell at the address, for a table the game's code indexes without a bound: outside
/// its region the program ends naming it.
#[track_caller]
pub fn cell_or_end(addr: u32, what: &str) -> &'static crate::session::Dword {
    watched(addr, 4, what);
    let backing =
        inside(addr, 4).unwrap_or_else(|| crate::fatal::raise::end(what, outside(addr, 4, what)));
    let cell = place_of(addr, backing) as *const crate::session::Dword;
    assert!(
        cell.is_aligned(),
        "{what}: {addr:#010x} is not a cell's place"
    );
    // SAFETY: the four bytes lie in a live region, aligned for the atomic (checked above);
    // `Dword` is a cell over a plain integer, so shared references to it may overlap writes.
    unsafe { &*cell }
}

/// A checked write, as `read_or_end`.
#[track_caller]
pub fn write_or_end<T: Plain>(addr: u32, value: T, what: &str) {
    if let Err(e) = GamePtr::new(addr).write(value, what) {
        crate::fatal::raise::end(what, e)
    }
}

/// The last region a run of reads found, kept for the next (a picture read per pixel): a
/// read inside it costs a compare, one outside looks its region up again. Keep a window for
/// one pass only: a block can be given back between passes.
#[derive(Default)]
pub struct Window(std::cell::Cell<(u32, u32, Backing)>);

impl Window {
    pub fn new() -> Self {
        Self::default()
    }

    #[track_caller]
    fn check(&self, addr: u32, len: usize, what: &str) -> Result<Backing, BadInput> {
        watched(addr, len, what);
        let end = addr.checked_add(len as u32);
        let (start, top, backing) = self.0.get();
        if addr >= start && end.is_some_and(|end| end <= top) {
            return Ok(backing);
        }
        match region_of(addr) {
            Some((start, top)) if end.is_some_and(|end| end <= top) => {
                let backing = backing_of(start);
                self.0.set((start, top, backing));
                Ok(backing)
            }
            _ => Err(outside(addr, len, what)),
        }
    }

    /// A checked read; outside its region the program ends (`fatal::raise::end`).
    pub fn read_or_end<T: Plain>(&self, addr: u32, what: &str) -> T {
        let mut value = zeroed::<T>();
        if let Err(e) = self
            .check(addr, size_of::<T>(), what)
            .and_then(|backing| load_at(addr, backing, bytes_mut(&mut value)))
        {
            crate::fatal::raise::end(what, e)
        }
        value
    }

    /// A checked write, as `read_or_end`.
    pub fn write_or_end<T: Plain>(&self, addr: u32, value: T, what: &str) {
        if let Err(e) = self
            .check(addr, size_of::<T>(), what)
            .and_then(|backing| store_at(addr, backing, bytes_of(&value)))
        {
            crate::fatal::raise::end(what, e)
        }
    }

    /// Checked bytes written, as `write_or_end`.
    #[track_caller]
    pub fn write_bytes_or_end(&self, addr: u32, bytes: &[u8], what: &str) {
        if let Err(e) = self
            .check(addr, bytes.len(), what)
            .and_then(|backing| store_at(addr, backing, bytes))
        {
            crate::fatal::raise::end(what, e)
        }
    }
}

/// Bytes owned here and reached by game address, for reading and for writing: a region for as
/// long as they live. They are held by a raw pointer, so a write by address is allowed
/// (a `Vec` lent through `&[u8]` gives read permission only: Miri, 2026-10-31).
pub struct LentBytes {
    block: &'static pool::PoolBlock,
    len: usize,
    game: u32,
}

impl LentBytes {
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        let block = pool::take(bytes.len());
        Store::store(block, 0, &bytes)
            .unwrap_or_else(|e| crate::fatal::raise::end("pointers::LentBytes", e));
        Self {
            block,
            len: bytes.len(),
            game: lend_block(block, bytes.len(), false),
        }
    }

    /// The bytes' game address.
    pub fn addr(&self) -> u32 {
        self.game
    }

    /// The bytes as a place bounded by them.
    pub fn ptr(&self) -> GamePtr {
        GamePtr::new(self.game).limited(self.len as u32)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// A copy of the bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = vec![0; self.len];
        Store::load(self.block, 0, &mut bytes)
            .unwrap_or_else(|e| crate::fatal::raise::end("pointers::LentBytes", e));
        bytes
    }

    /// The bytes in place.
    ///
    /// # Safety
    /// Nothing may write them while the slice lives.
    pub unsafe fn slice(&self) -> &[u8] {
        // SAFETY: the block holds `len` bytes; unwritten meanwhile (the caller's promise).
        unsafe { std::slice::from_raw_parts(self.block.as_ptr(), self.len) }
    }

    /// The bytes in place, to change.
    ///
    /// # Safety
    /// Nothing else may reach them by address while the slice lives.
    pub unsafe fn slice_mut(&mut self) -> &mut [u8] {
        // SAFETY: as `slice`.
        unsafe { std::slice::from_raw_parts_mut(self.block.as_ptr(), self.len) }
    }
}

impl Drop for LentBytes {
    fn drop(&mut self) {
        if let Some(block) = forget_block(self.game) {
            pool::give_back(block);
        }
    }
}

/// A Rust buffer lent to code that reads by the game's addresses: a region while the guard
/// lives; code that writes needs `LentBytes` or a pointer taken from `as_mut_ptr`. Take the guard before the address is handed on, and drop
/// it before the buffer: a struct holding both declares the guard first (fields drop in
/// order; freed memory still a region is memory the game may reach, and memory the heap
/// hands out again inside a stale range: 2026-10-26, a flaky test).
#[cfg(test)]
pub struct Lent(u32);

#[cfg(test)]
impl Lent {
    pub fn new(bytes: &[u8]) -> Self {
        Self(lend(bytes.as_ptr(), bytes.len()))
    }

    /// The buffer's game address.
    pub fn addr(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
impl Drop for Lent {
    fn drop(&mut self) {
        forget(self.0);
    }
}

#[cfg(test)]
mod cells_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::strings::bytes::Bytes;

    #[test]
    fn reads_answer_inside_a_region_only() {
        // On the heap: the stack is a region of its own. Lent with write permission, as
        // `LentBytes` lends (reads are atomics, which need it).
        let block = Box::into_raw(Box::new([7u8; 64]));
        let start = map(block, 64);
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
        crate::address_map::unmap(start);
        // SAFETY: `block` is the box above, its range given back.
        drop(unsafe { Box::from_raw(block) });
    }

    #[test]
    fn moved_stores_are_regions() {
        let font = crate::gfx::typefaces::FONTS.objects[0].addr();
        assert!(GamePtr::new(font).read::<[u8; 0x20]>("test").is_ok());
    }

    #[test]
    fn a_window_keeps_its_region() {
        let block = LentBytes::from_vec(vec![5u8; 32]);
        let start = block.addr();
        let window = Window::new();
        assert_eq!(window.read_or_end::<u8>(start, "test"), 5);
        assert_eq!((window.0.get().0, window.0.get().1), (start, start + 32));
        window.write_or_end(start + 28, 0x0102_0304u32, "test");
        assert_eq!(block.to_vec()[28], 4);
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
        let at = map(heap.as_ptr(), 8);
        assert!(GamePtr::new(at).read::<u8>("test").is_err());
        // Its range given back before the memory is (tests run side by side: a stale
        // range would hold the next block the heap hands out there).
        crate::address_map::unmap(at);
    }
}
