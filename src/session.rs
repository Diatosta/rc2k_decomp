//! Phase 5, the session core: the variables many modules and the game's threads share,
//! moved out of RAL.EXE's image (`moved.rs`). The screen's size and format, the window's
//! state (`g_windowActive`, `g_quitting`, ...), what the processor is and has, the window
//! handles, and the clocks the key thread and the menus keep.
//!
//! Each dword is an atomic (`Dword`, plain loads and stores as the game's code did: the
//! audio, key and network threads read them without a lock). The blocks keep the image's
//! layout, so that an address taken of a field is the field's place and a walk from one
//! variable to its neighbour reads the same neighbour (the vendor text of the processor,
//! three dwords; the surface format, six dwords; the Windows version record).

#![allow(unsafe_code)]

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, Ordering::Relaxed};

/// One of the game's dwords: an atomic cell the game's threads read and write.
#[repr(transparent)]
pub struct Dword(AtomicU32);

impl Dword {
    pub const fn new(value: u32) -> Self {
        Self(AtomicU32::new(value))
    }

    pub fn get(&self) -> u32 {
        self.0.load(Relaxed)
    }

    pub fn set(&self, value: u32) {
        self.0.store(value, Relaxed);
    }

    /// The atomic itself, for code that wants more than a load and a store.
    pub fn atomic(&self) -> &AtomicU32 {
        &self.0
    }

    /// Its place, for code that is handed an address (a Windows call, an `fild`).
    pub fn as_ptr(&self) -> *mut u32 {
        self.0.as_ptr()
    }

    pub fn addr(&self) -> u32 {
        crate::pointers::game_address(self.as_ptr())
    }

    /// The bits, for the x87 model's loads (as `Float::bits`).
    pub fn bits(&self) -> u32 {
        self.get()
    }

    /// Stores the bits, for the x87 model's stores (as `Float::set_bits`).
    pub fn set_bits(&self, bits: u32) {
        self.set(bits);
    }
}

/// The values of a run of dwords (a list or table of the game's, `[Dword; N]`).
pub fn read_all<const N: usize>(cells: &[Dword; N]) -> [u32; N] {
    std::array::from_fn(|k| cells[k].get())
}

/// Stores a run of dwords.
pub fn write_all<const N: usize>(cells: &[Dword; N], values: [u32; N]) {
    for (cell, value) in cells.iter().zip(values) {
        cell.set(value);
    }
}

/// The bits of a run of dwords or floats (`Cell32`), as the image's bytes read whole.
pub fn read_bits<C: Cell32, const N: usize>(cells: &[C; N]) -> [u32; N] {
    std::array::from_fn(|k| cells[k].bits())
}

/// Stores a run of dwords or floats by their bits.
pub fn write_bits<C: Cell32, const N: usize>(cells: &[C; N], values: [u32; N]) {
    for (cell, value) in cells.iter().zip(values) {
        cell.set_bits(value);
    }
}

/// The bits of the first `N` cells of a longer run (the original reads part of a table whole).
pub fn read_first<C: Cell32, const N: usize>(cells: &[C]) -> [u32; N] {
    assert!(N <= cells.len());
    std::array::from_fn(|k| cells[k].bits())
}

/// Stores `values` by their bits into the first cells of a run (the rest keep theirs).
pub fn write_first<C: Cell32>(cells: &[C], values: &[u32]) {
    assert!(values.len() <= cells.len());
    for (cell, &value) in cells.iter().zip(values) {
        cell.set_bits(value);
    }
}

/// Stores `values` into the first cells of a run of dwords (the rest keep their values).
pub fn write_prefix(cells: &[Dword], values: &[u32]) {
    assert!(values.len() <= cells.len());
    for (cell, &value) in cells.iter().zip(values) {
        cell.set(value);
    }
}

/// One of the game's floats: an atomic cell holding its bits (phase 7, typed state).
#[repr(transparent)]
pub struct Float(AtomicU32);

impl Float {
    /// From the float's bits (the image's first values are bits).
    pub const fn from_bits(bits: u32) -> Self {
        Self(AtomicU32::new(bits))
    }

    pub fn get(&self) -> f32 {
        f32::from_bits(self.0.load(Relaxed))
    }

    pub fn set(&self, value: f32) {
        self.0.store(value.to_bits(), Relaxed);
    }

    /// The bits, for the x87 model's loads (`x87_float::ld`).
    pub fn bits(&self) -> u32 {
        self.0.load(Relaxed)
    }

    /// Stores the bits, for the x87 model's stores (`x87_float::st`).
    pub fn set_bits(&self, bits: u32) {
        self.0.store(bits, Relaxed);
    }

    /// Its place (the tests).
    #[cfg(test)]
    pub fn addr(&self) -> u32 {
        crate::pointers::game_address(self.0.as_ptr())
    }
}

/// Cell `n` of `cells`, or the dword the original reads there: past the table's end it reads what
/// follows only inside `holder`, the store or record holding the table. The original reads
/// on anywhere; here the next thing in memory is another static, so that is refused.
pub fn read_on_in<H: crate::pointers::Cells, C: Cell32>(
    holder: &H,
    cells: &[C],
    n: usize,
    what: &str,
) -> Result<u32, crate::errors::BadInput> {
    match cells.get(n) {
        Some(cell) => Ok(cell.bits()),
        None => {
            let mut bytes = [0; 4];
            on_in(holder, cells, n, what, |at| {
                crate::pointers::load_cells(holder, at, &mut bytes)
            })?;
            Ok(u32::from_le_bytes(bytes))
        }
    }
}

/// `read_on_in`'s write.
pub fn write_on_in<H: crate::pointers::Cells, C: Cell32>(
    holder: &H,
    cells: &[C],
    n: usize,
    v: u32,
    what: &str,
) -> Result<(), crate::errors::BadInput> {
    match cells.get(n) {
        Some(cell) => {
            cell.set_bits(v);
            Ok(())
        }
        None => on_in(holder, cells, n, what, |at| {
            crate::pointers::store_cells(holder, at, &v.to_le_bytes())
        }),
    }
}

/// `access` on dword `n` of `cells`, placed from `holder`'s first byte (refused, and logged,
/// when that is not inside `holder`).
fn on_in<H, C>(
    holder: &H,
    cells: &[C],
    n: usize,
    what: &str,
    access: impl FnOnce(usize) -> Result<(), crate::errors::BadInput>,
) -> Result<(), crate::errors::BadInput> {
    let at = (cells.as_ptr() as usize)
        .wrapping_sub(std::ptr::from_ref(holder) as usize)
        .wrapping_add(n.wrapping_mul(4));
    access(at).inspect_err(|_| {
        crate::framework::log(&format!(
            "pointers: {what}: dword {n} at +{at:#x}, outside its holder of {:#x} bytes",
            size_of::<H>()
        ));
    })
}

/// `n` cells from `from` into `to`, one at a time forwards (a `rep movsd`: an overlap reads
/// what it has just written), each past its table's end only inside its holder.
pub fn copy_on_in<F: crate::pointers::Cells, T: crate::pointers::Cells, A: Cell32, B: Cell32>(
    (from_holder, from): (&F, &[A]),
    (to_holder, to): (&T, &[B]),
    n: usize,
    what: &str,
) -> Result<(), crate::errors::BadInput> {
    for k in 0..n {
        write_on_in(
            to_holder,
            to,
            k,
            read_on_in(from_holder, from, k, what)?,
            what,
        )?;
    }
    Ok(())
}

/// A 32-bit cell the x87 model loads and stores by its bits: a `Dword` or a `Float` (a
/// parameter that may be either, the damage model's).
pub trait Cell32 {
    fn bits(&self) -> u32;
    fn set_bits(&self, bits: u32);
}

/// A cell by reference (a list of `&dyn Cell32` is a run of cells: `math::cells`).
impl<T: Cell32 + ?Sized> Cell32 for &T {
    fn bits(&self) -> u32 {
        (**self).bits()
    }
    fn set_bits(&self, bits: u32) {
        (**self).set_bits(bits);
    }
}

/// The byte `n` of `cells` (a table of bytes held as dwords, little-endian), if inside.
pub fn byte_of(cells: &[Dword], n: usize) -> Option<u8> {
    cells.get(n / 4).map(|c| c.get().to_le_bytes()[n % 4])
}

impl Cell32 for Dword {
    fn bits(&self) -> u32 {
        self.get()
    }
    fn set_bits(&self, bits: u32) {
        self.set(bits);
    }
}

impl Cell32 for Float {
    fn bits(&self) -> u32 {
        self.bits()
    }
    fn set_bits(&self, bits: u32) {
        self.set_bits(bits);
    }
}

/// One of the game's words (16 bits): an atomic cell.
#[repr(transparent)]
pub struct Word(AtomicU16);

impl Word {
    pub const fn new(value: u16) -> Self {
        Self(AtomicU16::new(value))
    }

    pub fn get(&self) -> u16 {
        self.0.load(Relaxed)
    }

    pub fn set(&self, value: u16) {
        self.0.store(value, Relaxed);
    }
}

/// One of the game's bytes: an atomic cell (phase 7, typed state).
#[repr(transparent)]
pub struct Byte(AtomicU8);

impl Byte {
    pub const fn new(value: u8) -> Self {
        Self(AtomicU8::new(value))
    }

    pub fn get(&self) -> u8 {
        self.0.load(Relaxed)
    }

    pub fn set(&self, value: u8) {
        self.0.store(value, Relaxed);
    }

    /// The atomic itself, for code that wants more than a load and a store (a once-only guard).
    pub fn atomic(&self) -> &AtomicU8 {
        &self.0
    }
}

/// A buffer of the game's bytes (a table, a record handed on by address), shared between the
/// game's threads as the image's bytes were: each byte an atomic of its own, reached at that
/// width only (`shared_memory` copies byte by byte too), so no access mixes sizes.
#[repr(transparent)]
pub struct Bytes<const N: usize>([AtomicU8; N]);

impl<const N: usize> Bytes<N> {
    pub const fn new(value: [u8; N]) -> Self {
        Self(atomic_bytes(value))
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr().cast::<u8>().cast_mut()
    }

    #[cfg(test)]
    pub fn addr(&self) -> u32 {
        crate::pointers::game_address(self.as_ptr())
    }

    pub fn get(&self, index: usize) -> u8 {
        self.0[index].load(Relaxed)
    }

    pub fn set(&self, index: usize, value: u8) {
        self.0[index].store(value, Relaxed);
    }

    /// The byte at `index` as an atomic (an `xchg` on a key).
    pub fn atomic(&self, index: usize) -> &AtomicU8 {
        &self.0[index]
    }

    /// The little-endian dword at byte `offset` (any alignment, as the game's records are
    /// read): four byte reads, so another thread's write of it may be seen half done.
    pub fn get_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(std::array::from_fn(|i| self.get(offset + i)))
    }

    pub fn set_u32(&self, offset: usize, value: u32) {
        for (i, b) in value.to_le_bytes().into_iter().enumerate() {
            self.set(offset + i, b);
        }
    }

    /// All of it.
    pub fn read(&self) -> [u8; N] {
        std::array::from_fn(|i| self.get(i))
    }

    /// The bytes up to the first NUL (or all of them): a text kept in the buffer.
    pub fn text(&self) -> Vec<u8> {
        let all = self.read();
        let n = all.iter().position(|&c| c == 0).unwrap_or(N);
        all[..n].to_vec()
    }

    pub fn write(&self, value: &[u8; N]) {
        for (i, &b) in value.iter().enumerate() {
            self.set(i, b);
        }
    }

    /// `len` bytes from `offset` (`what` names them in a refusal past the end).
    pub fn read_at(
        &self,
        offset: usize,
        len: usize,
        what: &str,
    ) -> Result<Vec<u8>, crate::errors::BadInput> {
        self.range(offset, len, what)?;
        Ok((offset..offset + len).map(|i| self.get(i)).collect())
    }

    /// `data` written from `offset`.
    pub fn write_at(
        &self,
        offset: usize,
        data: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        self.range(offset, data.len(), what)?;
        for (i, &b) in data.iter().enumerate() {
            self.set(offset + i, b);
        }
        Ok(())
    }

    fn range(&self, offset: usize, len: usize, what: &str) -> Result<(), crate::errors::BadInput> {
        match offset.checked_add(len) {
            Some(end) if end <= N => Ok(()),
            _ => {
                crate::framework::log(&format!(
                    "session: {what}: {len} bytes at {offset:#x} of a {N:#x}-byte buffer"
                ));
                Err(crate::errors::BadInput("past the end of a buffer"))
            }
        }
    }
}

/// A dword at an address that is not a multiple of 4 (the audio block's `g_has3d`): four
/// byte atomics, as `Bytes`.
#[repr(transparent)]
pub struct Loose([AtomicU8; 4]);

impl Loose {
    pub const fn new(value: u32) -> Self {
        Self(atomic_bytes(value.to_le_bytes()))
    }

    pub fn get(&self) -> u32 {
        u32::from_le_bytes(std::array::from_fn(|i| self.0[i].load(Relaxed)))
    }

    pub fn set(&self, value: u32) {
        for (cell, b) in self.0.iter().zip(value.to_le_bytes()) {
            cell.store(b, Relaxed);
        }
    }
}

/// `value`'s bytes as atomics (a `const` initialiser of `Bytes` and `Loose`), in one move: a
/// loop over a table of the game's data is too slow for the compiler's evaluator.
const fn atomic_bytes<const N: usize>(value: [u8; N]) -> [AtomicU8; N] {
    use std::mem::ManuallyDrop;
    union Both<const N: usize> {
        plain: ManuallyDrop<[u8; N]>,
        atomic: ManuallyDrop<[AtomicU8; N]>,
    }
    // SAFETY: `AtomicU8` has `u8`'s size, alignment and bit validity, so the arrays are the
    // same bytes; the value is moved, not shared.
    ManuallyDrop::into_inner(unsafe {
        Both {
            plain: ManuallyDrop::new(value),
        }
        .atomic
    })
}

/// A record the game keeps as a plain struct (a voice, a mixer channel, a stream player, a
/// list of memory blocks, a caps record): copied out and in whole, or borrowed for the
/// code that works on it as the core's struct. Raw access under the original's discipline
/// (the audio thread's turn and the game thread's), as the image's bytes were shared.
#[repr(transparent)]
pub struct Shared<T>(UnsafeCell<T>);

// SAFETY: as the image's memory: raw access only, under the original's discipline.
unsafe impl<T> Sync for Shared<T> {}

impl<T: Copy> Shared<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    /// Whether the record is reached by dwords (4-aligned: each whole dword one atomic, as
    /// the original's dwords were whole) or byte by byte (a packed record).
    pub const WORDS: bool = align_of::<T>() >= 4;

    /// A copy of the record (a record's fields may be read half changed, as the original's;
    /// a 4-aligned record's dwords are each read whole).
    pub fn get(&self) -> T {
        let mut value = std::mem::MaybeUninit::<T>::uninit();
        // SAFETY: the record's own bytes, reached only at the widths `WORDS` picks (here and
        // in its cells, `pointers`); every byte of `value` is written before it is taken.
        unsafe {
            let to =
                std::slice::from_raw_parts_mut(value.as_mut_ptr().cast::<u8>(), size_of::<T>());
            let base = self.0.get().cast::<u8>();
            if Self::WORDS {
                crate::shared_memory::load_words(base, size_of::<T>(), 0, to);
            } else {
                crate::shared_memory::load(base, to.as_mut_ptr(), to.len());
            }
            value.assume_init()
        }
    }

    pub fn set(&self, value: T) {
        // SAFETY: as `get`'s; `value`'s bytes are only read.
        unsafe {
            let from = std::slice::from_raw_parts((&raw const value).cast::<u8>(), size_of::<T>());
            let base = self.0.get().cast::<u8>();
            if Self::WORDS {
                crate::shared_memory::store_words(base, size_of::<T>(), 0, from);
            } else {
                crate::shared_memory::store(base, from.as_ptr(), from.len());
            }
        }
    }

    /// The record changed by `change` (read, changed, written back).
    pub fn update(&self, change: impl FnOnce(&mut T)) {
        let mut record = self.get();
        change(&mut record);
        self.set(record);
    }
}

impl<T> Shared<T> {
    pub fn as_ptr(&self) -> *mut T {
        self.0.get()
    }

    pub fn addr(&self) -> u32 {
        crate::pointers::game_address(self.as_ptr())
    }
}

const fn zeros<const N: usize>() -> [Dword; N] {
    [const { Dword::new(0) }; N]
}

/// The screen: 0x4F494C .. 0x4F4984.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Screen {
    pub width: Dword,       // g_screenWidth
    pub height: Dword,      // g_screenHeight
    pub bits: Dword,        // g_screenBits
    pub bytes: Dword,       // g_screenBytesPerPixel
    pub row_bytes: Dword,   // g_screenRowBytes
    pub row_dwords: Dword,  // g_screenRowDwords
    pub depth_bits: Dword,  // dword_4F4964
    pub kind: Dword,        // dword_4F4968
    pub format: [Dword; 6], // g_surfaceFormat (a pixel format record, 24 bytes)
}

pub static SCREEN: Screen = Screen {
    width: Dword::new(0),
    height: Dword::new(0),
    bits: Dword::new(0),
    bytes: Dword::new(0),
    row_bytes: Dword::new(0),
    row_dwords: Dword::new(0),
    depth_bits: Dword::new(0),
    kind: Dword::new(0),
    format: zeros(),
};

pub const SCREEN_IDA: usize = 0x4F494C;
pub const SCREEN_LEN: usize = core::mem::size_of::<Screen>();

/// The window's state and the main loop's: 0x518370 .. 0x51839C.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Flags {
    pub f370: Dword,           // dword_518370
    pub window_active: Dword,  // g_windowActive
    pub window_focused: Dword, // g_windowFocused
    pub quitting: Dword,       // g_quitting
    pub lost: Dword,           // dword_518380
    pub no_task_switch: Dword, // dword_518384 (`notaskswitch`, a `var\ral.cfg` setting)
    pub keep_priority: Dword,  // g_keepPriority (`noprichange`, a setting)
    /// `dword_51838C` and `g_mainLoopStack`: only the main loop's, which needs neither.
    pub _gap_51838c: [Dword; 2],
    pub f394: Dword,   // dword_518394 (1 at start)
    pub paused: Dword, // dword_518398
}

pub static FLAGS: Flags = Flags {
    f370: Dword::new(0),
    window_active: Dword::new(0),
    window_focused: Dword::new(0),
    quitting: Dword::new(0),
    lost: Dword::new(0),
    no_task_switch: Dword::new(0),
    keep_priority: Dword::new(0),
    _gap_51838c: [Dword::new(0), Dword::new(0)],
    f394: Dword::new(1),
    paused: Dword::new(0),
};

pub const FLAGS_IDA: usize = 0x518370;
pub const FLAGS_LEN: usize = core::mem::size_of::<Flags>();

/// What the machine is and has, and the window handles: 0x5183A8 .. 0x5184A8.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Machine {
    pub has_mmx: Dword,          // g_hasMmx
    pub ignore_mmx: Dword,       // dword_5183AC (`ignoremmx`)
    pub has_sse: Dword,          // g_hasSse
    pub ignore_katmai: Dword,    // dword_5183B4 (`ignorekatmai`)
    pub has_3dnow: Dword,        // g_has3dNow
    pub ignore_3dnow: Dword,     // dword_5183BC (`ignoreamdk3d`)
    pub work: Dword,             // dword_5183C0
    pub work_float: Dword,       // flt_5183C4
    pub gap_c8: Dword,           // 0x5183C8
    pub scratch: Dword,          // g_soundScratch: the x87 code's scratch dword
    pub cpu_family: Dword,       // g_cpuFamily
    pub cpu_signature: Dword,    // g_cpuSignature
    pub cpu_vendor: [Dword; 3],  // g_cpuVendor: the vendor's text
    pub gap_e4: Dword,           // 0x5183E4
    pub cpu_mhz: Dword,          // g_cpuMhz
    pub has_fpu: Dword,          // g_hasFpu
    pub has_tsc: Dword,          // g_hasTsc
    pub has_cmov: Dword,         // g_hasCmov
    pub found_mmx: Dword,        // dword_5183F8
    pub found_sse: Dword,        // dword_5183FC
    pub found_3dnow: Dword,      // dword_518400
    pub keyboard_type: Dword,    // g_keyboardType (1 at start)
    pub version: [Dword; 37],    // VersionInformation (OSVERSIONINFOA, 148 bytes)
    pub exception_resume: Dword, // g_exceptionResume
    pub instance: Dword,         // hInstance
    pub window: Dword,           // hWnd
}

pub static MACHINE: Machine = Machine {
    has_mmx: Dword::new(0),
    ignore_mmx: Dword::new(0),
    has_sse: Dword::new(0),
    ignore_katmai: Dword::new(0),
    has_3dnow: Dword::new(0),
    ignore_3dnow: Dword::new(0),
    work: Dword::new(0),
    work_float: Dword::new(0),
    gap_c8: Dword::new(0),
    scratch: Dword::new(0),
    cpu_family: Dword::new(0),
    cpu_signature: Dword::new(0),
    cpu_vendor: zeros(),
    gap_e4: Dword::new(0),
    cpu_mhz: Dword::new(0),
    has_fpu: Dword::new(0),
    has_tsc: Dword::new(0),
    has_cmov: Dword::new(0),
    found_mmx: Dword::new(0),
    found_sse: Dword::new(0),
    found_3dnow: Dword::new(0),
    keyboard_type: Dword::new(1),
    version: zeros(),
    exception_resume: Dword::new(0),
    instance: Dword::new(0),
    window: Dword::new(0),
};

pub const MACHINE_IDA: usize = 0x5183A8;
pub const MACHINE_LEN: usize = core::mem::size_of::<Machine>();

/// The key thread's count and the clocks the menus keep: 0x518998 .. 0x5189A4.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Clocks {
    pub key_ticks: Dword,  // dword_518998: the key thread's turns
    pub frame: Dword,      // dword_51899C: the menus' clock (`update_frame_clock`)
    pub setup_time: Dword, // dword_5189A0
}

pub static CLOCKS: Clocks = Clocks {
    key_ticks: Dword::new(0),
    frame: Dword::new(0),
    setup_time: Dword::new(0),
};

pub const CLOCKS_IDA: usize = 0x518998;
pub const CLOCKS_LEN: usize = core::mem::size_of::<Clocks>();

/// The address of a field or an array of cells, for code that is handed an address.
pub fn address_of<T: ?Sized>(field: &T) -> u32 {
    crate::pointers::game_address(std::ptr::from_ref::<T>(field).cast::<u8>())
}

/// `N` bytes holding `runs` ((offset, bytes)) and zeros elsewhere.
pub const fn with_runs<const N: usize>(runs: &[(usize, &[u8])]) -> [u8; N] {
    let mut out = [0u8; N];
    let mut r = 0;
    while r < runs.len() {
        let (at, bytes) = runs[r];
        let mut i = 0;
        while i < bytes.len() {
            out[at + i] = bytes[i];
            i += 1;
        }
        r += 1;
    }
    out
}

/// A text the game keeps in place and may write into (a path whose ending the picture
/// loader swaps): a typed buffer (`Bytes<N>`), or one known only by its game address
/// (`GamePtr`, checked as game memory).
pub trait TextPlace {
    /// The bytes up to the NUL; refused when none comes before the place's end.
    fn text(&self, what: &str) -> Result<Vec<u8>, crate::errors::BadInput>;
    /// `bytes` written `at` bytes in.
    fn write_text_at(
        &self,
        at: usize,
        bytes: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput>;
}

impl<const N: usize> TextPlace for Bytes<N> {
    fn text(&self, _what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        let all = self.read();
        match all.iter().position(|&c| c == 0) {
            Some(n) => Ok(all[..n].to_vec()),
            None => Err(crate::errors::BadInput("a text with no end in its buffer")),
        }
    }

    fn write_text_at(
        &self,
        at: usize,
        bytes: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        self.write_at(at, bytes, what)
    }
}

/// A text of the program's own (a literal with its NUL): read only. The writes the
/// original could make into its copy in the image (a missing `.png`'s ending swapped) are
/// refused.
impl<const N: usize> TextPlace for [u8; N] {
    fn text(&self, _what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        match self.iter().position(|&c| c == 0) {
            Some(n) => Ok(self[..n].to_vec()),
            None => Err(crate::errors::BadInput("a text with no end in its buffer")),
        }
    }

    fn write_text_at(&self, _: usize, _: &[u8], _: &str) -> Result<(), crate::errors::BadInput> {
        Err(crate::errors::BadInput(
            "a write into one of the program's own texts",
        ))
    }
}

impl TextPlace for crate::pointers::GamePtr {
    fn text(&self, what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        crate::files::cstring(*self, what)
    }

    fn write_text_at(
        &self,
        at: usize,
        bytes: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        self.offset(at as u32).write_bytes(bytes, what)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;

    fn bytes_of<T>(block: &'static T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(block).cast::<u8>();
        (0..core::mem::size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_blocks_start_as_the_images_bytes() {
        assert_eq!(bytes_of(&SCREEN), image_bytes(SCREEN_IDA, SCREEN_LEN));
        assert_eq!(bytes_of(&FLAGS), image_bytes(FLAGS_IDA, FLAGS_LEN));
        assert_eq!(bytes_of(&MACHINE), image_bytes(MACHINE_IDA, MACHINE_LEN));
        assert_eq!(bytes_of(&CLOCKS), image_bytes(CLOCKS_IDA, CLOCKS_LEN));
    }

    #[test]
    fn the_blocks_keep_the_images_layout() {
        use core::mem::offset_of;
        assert_eq!(SCREEN_LEN, 0x4F4984 - 0x4F494C);
        assert_eq!(offset_of!(Screen, format), 0x4F496C - 0x4F494C);
        assert_eq!(FLAGS_LEN, 0x51839C - 0x518370);
        assert_eq!(offset_of!(Flags, quitting), 0x51837C - 0x518370);
        assert_eq!(offset_of!(Flags, paused), 0x518398 - 0x518370);
        assert_eq!(MACHINE_LEN, 0x5184A8 - 0x5183A8);
        assert_eq!(offset_of!(Machine, scratch), 0x5183CC - 0x5183A8);
        assert_eq!(offset_of!(Machine, cpu_vendor), 0x5183D8 - 0x5183A8);
        assert_eq!(offset_of!(Machine, cpu_mhz), 0x5183E8 - 0x5183A8);
        assert_eq!(offset_of!(Machine, found_3dnow), 0x518400 - 0x5183A8);
        assert_eq!(offset_of!(Machine, version), 0x518408 - 0x5183A8);
        assert_eq!(offset_of!(Machine, exception_resume), 0x51849C - 0x5183A8);
        assert_eq!(offset_of!(Machine, window), 0x5184A4 - 0x5183A8);
        assert_eq!(CLOCKS_LEN, 12);
    }
}
