//! The game's own pointers (the 32-bit addresses its data holds), checked against what is
//! there.
//!
//! Every block the game takes from the system (`memory::allocate`)
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
//! Rust's own memory is not a region: its
//! statics are reached as stores or as values, its locals as slices or values, so a game
//! address into the program's sections or a thread's stack is refused, naming its caller.
//! Tests alone keep the coarse regions (each section of the image, the running thread's
//! stack), for the tests' references to the original's code, which take their buffers by
//! address.

#![allow(unsafe_code)]

use crate::errors::BadInput;
use crate::framework::log;
use std::collections::BTreeMap;
use std::sync::RwLock;

pub mod pool;

pub use pool::PoolBytes;

pub use crate::cells::{
    CellRun, Cells, Plain, Store, StoreBytes, cells_of_bytes, copy_between, copy_cells, dwords_in,
    load_cells, load_value, store_cells, store_value,
};
// Private helpers of the cells, used by the address layer and `pool`.
use crate::cells::{bytes_mut, bytes_of, zeroed};

/// The blocks taken from the system: start -> length.
static BLOCKS: RwLock<BTreeMap<u32, u32>> = RwLock::new(BTreeMap::new());

/// A block taken from the system (`len` bytes from `start`, header included).
#[cfg(test)]
pub fn register(start: u32, len: u32) {
    if let Ok(mut blocks) = BLOCKS.write() {
        blocks.insert(start, len);
    }
}

/// A block given back.
#[cfg(test)]
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

/// Stores that are regions of their own, by their game address: the drawing threads' copies
/// of the drawing's state (`workers`; copy 0 is the store's home, `moved`).
#[cfg(test)]
static STORE_HOMES: RwLock<BTreeMap<u32, &'static (dyn Store + Sync)>> =
    RwLock::new(BTreeMap::new());

/// `store` a region of its own, reached through its cells as a moved store's home is.
#[cfg(test)]
pub fn register_store(store: &'static (dyn Store + Sync)) {
    let at = game_address(store.first());
    if let Ok(mut homes) = STORE_HOMES.write() {
        homes.insert(at, store);
    }
    register(at, store.size() as u32);
}

/// The backing of the region that starts at `start`.
fn backing_of(start: u32) -> Backing {
    #[cfg(test)]
    if let Some(&store) = STORE_HOMES.read().ok().as_ref().and_then(|h| h.get(&start)) {
        return Backing::Store(store, start);
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
#[cfg(test)]
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

/// Lent memory given back: its region and its game range.
#[cfg(test)]
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
/// Ranges of game memory watched while a store moves out of them: each access
/// through the checked paths (`GamePtr`, `check_or_end`, `read_or_end`, `RegionCache`) that
/// touches one is logged once per source line of its first caller outside this file (with
/// `#[track_caller]` up to there) and then goes on as before, so one run lists every
/// reader by address. Up to four: (start, end, name).
static WATCHED: [std::sync::atomic::AtomicU64; 4] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 4];
static WATCH_NAMES: std::sync::Mutex<[&'static str; 4]> = std::sync::Mutex::new([""; 4]);
/// The (file, line, range) already logged.
type Seen = std::collections::HashSet<(&'static str, u32, &'static str)>;
static SEEN: std::sync::Mutex<Option<Seen>> = std::sync::Mutex::new(None);

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
/// network's mode). A place the original reads on past a field is a store's bytes
/// (`place::BytePlace::in_store`).
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

#[cfg(test)]
impl Eq for GamePtr {}

/// By the address, as `PartialEq` (a place's field ends do not make it another place).
#[cfg(test)]
impl std::hash::Hash for GamePtr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.at.hash(state);
    }
}

impl GamePtr {
    pub const fn new(addr: u32) -> Self {
        Self {
            at: addr,
            field: (0, 0),
        }
    }

    /// The game address of a field of Rust's game state (a store's cell, buffer or place).
    #[cfg(test)]
    pub fn to<T: ?Sized>(field: &T) -> Self {
        let at = game_address(std::ptr::from_ref::<T>(field).cast::<u8>());
        Self {
            at,
            field: (at, at.wrapping_add(size_of_val(field) as u32)),
        }
    }

    /// A store's bytes as a place bounded by them.
    #[cfg(test)]
    pub fn of_store(store: &dyn Store) -> Self {
        let at = game_address(store.first());
        Self {
            at,
            field: (at, at.wrapping_add(store.size() as u32)),
        }
    }

    #[cfg(test)]
    pub const fn addr(self) -> u32 {
        self.at
    }

    /// The same address, limited to `len` bytes from here: a buffer that is part of a field.
    #[cfg(test)]
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

    /// The `T` at the address (unaligned, as the game's data may be), when all of it lies in
    /// one region.
    #[track_caller]
    #[cfg(test)]
    pub fn read<T: Plain>(self, what: &str) -> Result<T, BadInput> {
        let backing = self.check(size_of::<T>(), what)?;
        let mut value = zeroed::<T>();
        load_at(self.at, backing, bytes_mut(&mut value))?;
        Ok(value)
    }

    /// The `T` written at the address, when all of it lies in one region.
    #[cfg(test)]
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

    /// The record of cells at the address (`Cells`: atomics and cells only, so a shared
    /// reference to memory the game also writes is sound), when all of it lies in one region
    /// and the address has the record's alignment. Keep it no longer than the step that
    /// reads it: a block can be given back later.
    #[cfg(test)]
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

/// `to.len()` bytes from the game address `addr`, which the caller checked to lie in one
/// region (`backing`, what holds it): through its store when a moved range holds it, else the region's raw bytes (the
/// one raw read left: blocks from the system, Rust memory lent by address).
fn load_at(addr: u32, backing: Backing, to: &mut [u8]) -> Result<(), BadInput> {
    if let Backing::Store(store, home) = backing {
        let at = (addr - home) as usize;
        crate::write_watch::by_address(store, at, to.len(), false);
        return match run_of(store, at, to.len()) {
            Some((start, run)) => {
                run.load(at - start, to);
                Ok(())
            }
            None => store.load(at, to),
        };
    }
    raw_load(addr, to);
    Ok(())
}

/// `from` written at the game address `addr`, as `load_at` reads.
fn store_at(addr: u32, backing: Backing, from: &[u8]) -> Result<(), BadInput> {
    if let Backing::Store(store, home) = backing {
        let at = (addr - home) as usize;
        crate::write_watch::by_address(store, at, from.len(), true);
        return match run_of(store, at, from.len()) {
            Some((start, run)) => {
                run.store(at - start, from);
                Ok(())
            }
            None => store.store(at, from),
        };
    }
    raw_store(addr, from);
    Ok(())
}

/// Where `addr`'s bytes are in the region `backing` holds (checked by the caller): in its
/// store's bytes, from the store's first one.
#[cfg(test)]
fn place_of(addr: u32, backing: Backing) -> *mut u8 {
    match backing {
        Backing::Store(store, home) => store
            .first()
            .cast_mut()
            .wrapping_add((addr - home) as usize),
        Backing::Raw => raw_place(addr),
    }
}

/// The raw regions' copies (checked by the callers to lie in one live region): the tests'
/// only; in the program a raw region is refused (`raw_place`).
#[cfg(test)]
fn raw_load(addr: u32, to: &mut [u8]) {
    // SAFETY: the run lies in one live region (the caller's check); `to` is Rust's.
    unsafe { crate::shared_memory::load(raw_place(addr), to.as_mut_ptr(), to.len()) };
}

#[cfg(test)]
fn raw_store(addr: u32, from: &[u8]) {
    // SAFETY: as `raw_load`'s; the regions are the game's writable memory.
    unsafe { crate::shared_memory::store(raw_place(addr), from.as_ptr(), from.len()) };
}

#[cfg(not(test))]
fn raw_load(addr: u32, _: &mut [u8]) {
    raw_place(addr);
}

#[cfg(not(test))]
fn raw_store(addr: u32, _: &[u8]) {
    raw_place(addr);
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

/// The run of like cells in `store` holding `len` bytes at `at`, and the byte it starts at:
/// the thread's last run (most accesses step through one table), else its run looked up.
fn run_of(store: &'static dyn Store, at: usize, len: usize) -> Option<(usize, CellRun<'static>)> {
    thread_local! {
        static LAST: std::cell::Cell<Option<(*const u8, usize, CellRun<'static>)>> =
            const { std::cell::Cell::new(None) };
    }
    let first = store.first();
    let (start, run) = match LAST.get() {
        Some((was, start, run)) if was == first && at >= start && at < start + run.len() => {
            (start, run)
        }
        _ => {
            let (start, run) = store.cell_run_at(at)?;
            LAST.set(Some((first, start, run)));
            (start, run)
        }
    };
    (at + len <= start + run.len()).then_some((start, run))
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
    fn copy_run(&self, at: usize, out: &mut [u8]) -> Option<()> {
        let end = at.checked_add(out.len())?;
        // Inside the region (checked against its length): one read for all of it.
        (end <= self.len && load_at(self.start + at as u32, self.backing, out).is_ok())
            .then_some(())
    }
}

/// `len` bytes from `addr` checked to lie in one region; outside, the program ends naming
/// them (`fatal::raise::end`). For hot paths (the x87 runs' operands).
#[track_caller]
#[cfg(all(test, target_arch = "x86"))] // only the 32-bit tests' kept x87 runs use it
pub fn check_or_end(addr: u32, len: usize, what: &str) {
    watched(addr, len, what);
    match inside(addr, len) {
        Some(backing) => note_by_address(addr, backing, len, false),
        None => crate::fatal::raise::end(what, outside(addr, len, what)),
    }
}

/// An access by address to the store behind `backing`, noted for the field-leaving census
/// (`write_watch`).
#[cfg(all(test, target_arch = "x86"))]
#[inline(always)]
#[cfg_attr(feature = "write_watch", track_caller)]
fn note_by_address(_addr: u32, _backing: Backing, _len: usize, _write: bool) {
    #[cfg(feature = "write_watch")]
    if let Backing::Store(store, home) = _backing {
        crate::write_watch::by_address(store, (_addr - home) as usize, _len, _write);
    }
}

/// A checked read where the caller has no way to carry a refusal (generated step code, a
/// core's closure): outside its region the program ends naming it (`fatal::raise::end`).
#[track_caller]
#[cfg(test)]
pub fn read_or_end<T: Plain>(addr: u32, what: &str) -> T {
    GamePtr::new(addr)
        .read(what)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e))
}

/// A checked write, as `read_or_end`.
#[cfg(test)]
#[track_caller]
pub fn write_or_end<T: Plain>(addr: u32, value: T, what: &str) {
    if let Err(e) = GamePtr::new(addr).write(value, what) {
        crate::fatal::raise::end(what, e)
    }
}

/// A Rust buffer lent to code that reads by the game's addresses: a region while the guard
/// lives; code that writes needs a pointer taken from `as_mut_ptr`. Take the guard before the address is handed on, and drop
/// it before the buffer: a struct holding both declares the guard first (fields drop in
/// order; freed memory still a region is memory the game may reach, and memory the heap
/// hands out again inside a stale range, which made a test flaky).
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
mod tests {
    use super::*;
    use crate::gfx::owner::gfx;
    use rc2k_core::strings::bytes::Bytes;

    #[test]
    fn reads_answer_inside_a_region_only() {
        // On the heap: the stack is a region of its own. Lent with write permission
        // (reads are atomics, which need it).
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
        let font = gfx().fonts.typefaces.objects[0].addr();
        assert!(GamePtr::new(font).read::<[u8; 0x20]>("test").is_ok());
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
