//! The session core: the variables many modules and the game's threads share,
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

#[cfg(test)]
use crate::system::owner::system;
use std::cell::{Cell, UnsafeCell};
use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, Ordering::Relaxed};

/// One of the game's dwords: an atomic cell the game's threads read and write.
#[repr(transparent)]
pub struct Dword(AtomicU32);

impl Dword {
    pub const fn new(value: u32) -> Self {
        Self(AtomicU32::new(value))
    }

    pub fn get(&self) -> u32 {
        crate::write_watch::read(self.0.as_ptr().cast::<u8>(), 4);
        self.0.load(Relaxed)
    }

    pub fn set(&self, value: u32) {
        crate::write_watch::wrote(self.0.as_ptr().cast::<u8>(), 4);
        self.0.store(value, Relaxed);
    }

    /// Whether the cell, used as a flag, is not 0.
    pub fn is_set(&self) -> bool {
        self.get() != 0
    }

    /// The atomic itself, for code that wants more than a load and a store.
    pub fn atomic(&self) -> &AtomicU32 {
        &self.0
    }

    /// The cell's 32 bits (as `Float::bits`).
    pub fn bits(&self) -> u32 {
        self.get()
    }

    /// Stores the 32 bits (as `Float::set_bits`).
    pub fn set_bits(&self, bits: u32) {
        self.set(bits);
    }
}

/// The values of a run of dwords (a list or table of the game's, `[Dword; N]`).
pub fn read_all<C: Cell32, const N: usize>(cells: &[C; N]) -> [u32; N] {
    std::array::from_fn(|k| cells[k].bits())
}

/// Stores a run of dwords.
pub fn write_all<C: Cell32, const N: usize>(cells: &[C; N], values: [u32; N]) {
    for (cell, value) in cells.iter().zip(values) {
        cell.set_bits(value);
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
pub fn write_prefix<C: Cell32>(cells: &[C], values: &[u32]) {
    assert!(values.len() <= cells.len());
    for (cell, &value) in cells.iter().zip(values) {
        cell.set_bits(value);
    }
}

/// One of the game's floats: an atomic cell holding its bits.
#[repr(transparent)]
pub struct Float(AtomicU32);

impl Float {
    /// A cell holding `value`.
    pub const fn new(value: f32) -> Self {
        Self(AtomicU32::new(value.to_bits()))
    }

    /// From the float's bits (the image's first values are bits).
    pub const fn from_bits(bits: u32) -> Self {
        Self(AtomicU32::new(bits))
    }

    pub fn get(&self) -> f32 {
        crate::write_watch::read(self.0.as_ptr().cast::<u8>(), 4);
        f32::from_bits(self.0.load(Relaxed))
    }

    pub fn set(&self, value: f32) {
        crate::write_watch::wrote(self.0.as_ptr().cast::<u8>(), 4);
        self.0.store(value.to_bits(), Relaxed);
    }

    /// The atomic holding its bits (the cells' copies, `cells::CellRef`).
    pub fn atomic(&self) -> &AtomicU32 {
        &self.0
    }

    /// The cell's 32 bits (read by `x87_float::ld`).
    pub fn bits(&self) -> u32 {
        crate::write_watch::read(self.0.as_ptr().cast::<u8>(), 4);
        self.0.load(Relaxed)
    }

    /// Stores the 32 bits (written by `x87_float::st`).
    pub fn set_bits(&self, bits: u32) {
        crate::write_watch::wrote(self.0.as_ptr().cast::<u8>(), 4);
        self.0.store(bits, Relaxed);
    }
}

/// Cell `n` of `cells`, or the dword the original reads there: past the table's end it reads what
/// follows only inside `holder`, the store or record holding the table. The original reads
/// on anywhere; here the next thing in memory is another static, so that is refused.
pub fn read_on_in<H: crate::cells::Cells, C: Cell32>(
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
                crate::cells::load_cells(holder, at, &mut bytes)
            })?;
            Ok(u32::from_le_bytes(bytes))
        }
    }
}

/// `read_on_in`'s write.
pub fn write_on_in<H: crate::cells::Cells, C: Cell32>(
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
            crate::cells::store_cells(holder, at, &v.to_le_bytes())
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
pub fn copy_on_in<F: crate::cells::Cells, T: crate::cells::Cells, A: Cell32, B: Cell32>(
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

/// A 32-bit cell read and written by its bits (the float code's loads and stores): a `Dword` or a
/// `Float` (a parameter that may be either, the damage model's).
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
pub fn byte_of<C: Cell32>(cells: &[C], n: usize) -> Option<u8> {
    cells.get(n / 4).map(|c| c.bits().to_le_bytes()[n % 4])
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

/// One of the game's 0 / 1 dwords (a loop's "done" flag, "in the race"): the same four bytes as
/// a `Dword`, read as a truth value. Only 0 and 1 are ever stored.
#[repr(transparent)]
pub struct Flag(AtomicU32);

impl Flag {
    pub const fn new(value: bool) -> Self {
        Self(AtomicU32::new(value as u32))
    }

    pub fn is_set(&self) -> bool {
        crate::write_watch::read(self.0.as_ptr().cast::<u8>(), 4);
        self.0.load(Relaxed) != 0
    }

    pub fn set(&self, value: bool) {
        crate::write_watch::wrote(self.0.as_ptr().cast::<u8>(), 4);
        self.0.store(value as u32, Relaxed);
    }

    /// The atomic itself, for code that wants more than a load and a store.
    pub fn atomic(&self) -> &AtomicU32 {
        &self.0
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
        crate::write_watch::read(self.0.as_ptr().cast::<u8>(), 2);
        self.0.load(Relaxed)
    }

    pub fn set(&self, value: u16) {
        crate::write_watch::wrote(self.0.as_ptr().cast::<u8>(), 2);
        self.0.store(value, Relaxed);
    }

    /// The atomic itself (the cells' copies, `cells::CellRef`).
    pub fn atomic(&self) -> &AtomicU16 {
        &self.0
    }
}

/// One of the game's bytes: an atomic cell.
#[repr(transparent)]
pub struct Byte(AtomicU8);

impl Byte {
    pub const fn new(value: u8) -> Self {
        Self(AtomicU8::new(value))
    }

    pub fn get(&self) -> u8 {
        crate::write_watch::read(self.0.as_ptr().cast::<u8>(), 1);
        self.0.load(Relaxed)
    }

    pub fn set(&self, value: u8) {
        crate::write_watch::wrote(self.0.as_ptr().cast::<u8>(), 1);
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

    #[cfg(test)]
    pub fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr().cast::<u8>().cast_mut()
    }

    /// Its atomics (the cells' copies, `cells::CellRef`).
    pub fn atomics(&self) -> &[AtomicU8] {
        &self.0
    }

    pub fn get(&self, index: usize) -> u8 {
        crate::write_watch::read(self.0[index].as_ptr().cast_const(), 1);
        self.0[index].load(Relaxed)
    }

    pub fn set(&self, index: usize, value: u8) {
        crate::write_watch::wrote(self.0[index].as_ptr().cast_const(), 1);
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
        crate::write_watch::read(self.0[0].as_ptr().cast_const(), 4);
        u32::from_le_bytes(std::array::from_fn(|i| self.0[i].load(Relaxed)))
    }

    pub fn set(&self, value: u32) {
        crate::write_watch::wrote(self.0[0].as_ptr().cast_const(), 4);
        for (cell, b) in self.0.iter().zip(value.to_le_bytes()) {
            cell.store(b, Relaxed);
        }
    }

    /// Its atomics (the cells' copies, `cells::CellRef`).
    pub fn atomics(&self) -> &[AtomicU8] {
        &self.0
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

/// One value only the game's thread reaches: a non-atomic cell (`!Sync`, so that a second
/// thread's access does not compile). A store of these sits in a `GameThread`.
#[allow(dead_code, reason = "the next stores to flip (M3) use it")]
#[repr(transparent)]
pub struct Plain<T: Copy>(Cell<T>);

#[allow(dead_code, reason = "the next stores to flip (M3) use it")]
impl<T: Copy> Plain<T> {
    pub const fn new(value: T) -> Self {
        Self(Cell::new(value))
    }

    pub fn get(&self) -> T {
        self.0.get()
    }

    pub fn set(&self, value: T) {
        plain_written(&self.0);
        self.0.set(value);
    }

    /// The value changed by `change`.
    pub fn update(&self, change: impl FnOnce(T) -> T) {
        plain_written(&self.0);
        self.0.set(change(self.0.get()));
    }
}

/// A `Dword` only the game's thread reaches: the same four bytes and the same methods, no
/// atomic and no write watch.
#[repr(transparent)]
pub struct PlainDword(Cell<u32>);

#[allow(dead_code, reason = "the next stores to flip (M3) use it")]
impl PlainDword {
    pub const fn new(value: u32) -> Self {
        Self(Cell::new(value))
    }

    pub fn get(&self) -> u32 {
        self.0.get()
    }

    pub fn set(&self, value: u32) {
        plain_written(&self.0);
        self.0.set(value);
    }

    /// Whether the cell, used as a flag, is not 0.
    pub fn is_set(&self) -> bool {
        self.0.get() != 0
    }

    /// Its place, for code that is handed an address (a Windows call, an `fild`).
    #[cfg(test)]
    pub fn as_ptr(&self) -> *mut u32 {
        self.0.as_ptr()
    }

    /// The cell's 32 bits (as `PlainFloat::bits`).
    pub fn bits(&self) -> u32 {
        self.0.get()
    }

    /// Stores the 32 bits (as `PlainFloat::set_bits`).
    pub fn set_bits(&self, bits: u32) {
        plain_written(&self.0);
        self.0.set(bits);
    }

    /// The cell itself (the cells' copies of a run).
    pub(crate) fn cell(&self) -> &Cell<u32> {
        &self.0
    }
}

/// A `Float` only the game's thread reaches, holding its bits.
#[repr(transparent)]
pub struct PlainFloat(Cell<u32>);

#[allow(dead_code, reason = "the next stores to flip (M3) use it")]
impl PlainFloat {
    /// A cell holding `value`.
    pub const fn new(value: f32) -> Self {
        Self(Cell::new(value.to_bits()))
    }

    /// From the float's bits (the image's first values are bits).
    pub const fn from_bits(bits: u32) -> Self {
        Self(Cell::new(bits))
    }

    pub fn get(&self) -> f32 {
        f32::from_bits(self.0.get())
    }

    pub fn set(&self, value: f32) {
        plain_written(&self.0);
        self.0.set(value.to_bits());
    }

    /// The cell's 32 bits (read by `x87_float::ld`).
    pub fn bits(&self) -> u32 {
        self.0.get()
    }

    /// Stores the 32 bits (written by `x87_float::st`).
    pub fn set_bits(&self, bits: u32) {
        plain_written(&self.0);
        self.0.set(bits);
    }

    /// The cell itself (the cells' copies of a run).
    pub(crate) fn cell(&self) -> &Cell<u32> {
        &self.0
    }
}

/// A `Flag` only the game's thread reaches: 0 or 1 in four bytes.
#[repr(transparent)]
pub struct PlainFlag(Cell<u32>);

#[allow(dead_code, reason = "the next stores to flip (M3) use it")]
impl PlainFlag {
    pub const fn new(value: bool) -> Self {
        Self(Cell::new(value as u32))
    }

    pub fn is_set(&self) -> bool {
        self.0.get() != 0
    }

    pub fn set(&self, value: bool) {
        plain_written(&self.0);
        self.0.set(value as u32);
    }

    /// The cell itself (the cells' copies of a run).
    pub(crate) fn cell(&self) -> &Cell<u32> {
        &self.0
    }
}

impl Cell32 for PlainDword {
    fn bits(&self) -> u32 {
        self.0.get()
    }
    fn set_bits(&self, bits: u32) {
        plain_written(&self.0);
        self.0.set(bits);
    }
}

impl Cell32 for PlainFloat {
    fn bits(&self) -> u32 {
        self.0.get()
    }
    fn set_bits(&self, bits: u32) {
        plain_written(&self.0);
        self.0.set(bits);
    }
}

/// A `Word` only the game's thread reaches: the same two bytes and methods, no atomic.
#[repr(transparent)]
pub struct PlainWord(Cell<u16>);

impl PlainWord {
    pub const fn new(value: u16) -> Self {
        Self(Cell::new(value))
    }

    pub fn get(&self) -> u16 {
        self.0.get()
    }

    pub fn set(&self, value: u16) {
        plain_written(&self.0);
        self.0.set(value);
    }

    /// The cell itself (the cells' copies of a run).
    pub(crate) fn cell(&self) -> &Cell<u16> {
        &self.0
    }
}

/// A `Byte` only the game's thread reaches.
#[repr(transparent)]
pub struct PlainByte(Cell<u8>);

impl PlainByte {
    pub const fn new(value: u8) -> Self {
        Self(Cell::new(value))
    }

    pub fn get(&self) -> u8 {
        self.0.get()
    }

    pub fn set(&self, value: u8) {
        plain_written(&self.0);
        self.0.set(value);
    }

    /// The cell itself (the cells' copies of a run).
    pub(crate) fn cell(&self) -> &Cell<u8> {
        &self.0
    }
}

/// A buffer of the game's bytes only the game's thread reaches: the same methods as `Bytes`,
/// each byte a plain cell.
#[repr(transparent)]
pub struct PlainBytes<const N: usize>([Cell<u8>; N]);

impl<const N: usize> PlainBytes<N> {
    pub const fn new(value: [u8; N]) -> Self {
        Self(plain_bytes(value))
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr().cast::<u8>().cast_mut()
    }

    /// Its cells (the cells' copies, `cells::CellRef`).
    pub(crate) fn cells(&self) -> &[Cell<u8>] {
        &self.0
    }

    pub fn get(&self, index: usize) -> u8 {
        self.0[index].get()
    }

    pub fn set(&self, index: usize, value: u8) {
        plain_written(&self.0[index]);
        self.0[index].set(value);
    }

    /// The little-endian dword at byte `offset` (any alignment, as the game's records are read).
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

/// A `Loose` only the game's thread reaches: a dword at an address that is not a multiple of
/// 4, four byte cells.
#[repr(transparent)]
pub struct PlainLoose([Cell<u8>; 4]);

impl PlainLoose {
    pub const fn new(value: u32) -> Self {
        Self(plain_bytes(value.to_le_bytes()))
    }

    pub fn get(&self) -> u32 {
        u32::from_le_bytes(std::array::from_fn(|i| self.0[i].get()))
    }

    pub fn set(&self, value: u32) {
        for (cell, b) in self.0.iter().zip(value.to_le_bytes()) {
            plain_written(cell);
            cell.set(b);
        }
    }

    /// Its cells (the cells' copies, `cells::CellRef`).
    pub(crate) fn cells(&self) -> &[Cell<u8>] {
        &self.0
    }
}

/// `value`'s bytes as cells (a `const` initialiser of `PlainBytes` and `PlainLoose`), in one
/// move: a loop over a table of the game's data is too slow for the compiler's evaluator.
const fn plain_bytes<const N: usize>(value: [u8; N]) -> [Cell<u8>; N] {
    use std::mem::ManuallyDrop;
    union Both<const N: usize> {
        plain: ManuallyDrop<[u8; N]>,
        cells: ManuallyDrop<[Cell<u8>; N]>,
    }
    // SAFETY: `Cell<u8>` has `u8`'s size, alignment and bit validity, so the arrays are the
    // same bytes; the value is moved, not shared.
    ManuallyDrop::into_inner(unsafe {
        Both {
            plain: ManuallyDrop::new(value),
        }
        .cells
    })
}

/// A dword cell read and written as a number, whether shared (`Dword`) or the game thread's
/// (`PlainDword`): what the menus' and the screens' tables name when their cells come from
/// both kinds of store.
pub trait DwordLike {
    fn get(&self) -> u32;
    fn set(&self, value: u32);
}

impl DwordLike for Dword {
    fn get(&self) -> u32 {
        Dword::get(self)
    }
    fn set(&self, value: u32) {
        Dword::set(self, value);
    }
}

impl DwordLike for PlainDword {
    fn get(&self) -> u32 {
        PlainDword::get(self)
    }
    fn set(&self, value: u32) {
        PlainDword::set(self, value);
    }
}

/// A flag cell, shared (`Flag`) or the game thread's (`PlainFlag`).
pub trait FlagLike {
    fn is_set(&self) -> bool;
    fn set(&self, value: bool);
}

impl FlagLike for Flag {
    fn is_set(&self) -> bool {
        Flag::is_set(self)
    }
    fn set(&self, value: bool) {
        Flag::set(self, value);
    }
}

impl FlagLike for PlainFlag {
    fn is_set(&self) -> bool {
        PlainFlag::is_set(self)
    }
    fn set(&self, value: bool) {
        PlainFlag::set(self, value);
    }
}

/// A float cell read and written as a number, shared (`Float`) or the game thread's
/// (`PlainFloat`); its bits through `Cell32`.
pub trait FloatLike: Cell32 {
    fn get(&self) -> f32;
    fn set(&self, value: f32);
}

impl FloatLike for Float {
    fn get(&self) -> f32 {
        Float::get(self)
    }
    fn set(&self, value: f32) {
        Float::set(self, value);
    }
}

impl FloatLike for PlainFloat {
    fn get(&self) -> f32 {
        PlainFloat::get(self)
    }
    fn set(&self, value: f32) {
        PlainFloat::set(self, value);
    }
}

/// The kind of cells a record is made of, for a record kept in both kinds: the game thread's
/// plain cells (`PlainCells`, the stores) or atomics (`SharedCells`, a copy other threads
/// reach, such as a view's copy of its car). The two kinds of one record have the same
/// layout and first values, and copy into each other cell by cell (`cells::copy_cells`).
pub trait CellKind {
    type Dword: crate::cells::Cells + DwordLike + Cell32;
    type Float: crate::cells::Cells + FloatLike;
    type Bytes<const N: usize>: crate::cells::Cells;
}

/// The game thread's cells (`PlainDword`, `PlainFloat`, `PlainBytes`).
pub struct PlainCells;

impl CellKind for PlainCells {
    type Dword = PlainDword;
    type Float = PlainFloat;
    type Bytes<const N: usize> = PlainBytes<N>;
}

/// Atomic cells (`Dword`, `Float`, `Bytes`).
pub struct SharedCells;

impl CellKind for SharedCells {
    type Dword = Dword;
    type Float = Float;
    type Bytes<const N: usize> = Bytes<N>;
}

static GAME_THREAD: std::sync::OnceLock<std::thread::ThreadId> = std::sync::OnceLock::new();

/// Names the calling thread as the game's thread: from now on a debug build checks that a
/// `GameThread` is reached from it only. Called once, first thing on the main thread; the
/// tests never call it (each test runs on a thread of its own).
pub fn set_game_thread() {
    let _ = GAME_THREAD.set(std::thread::current().id());
}

/// Debug builds: panics when the game's thread is named and this is another one.
#[inline]
pub(crate) fn assert_game_thread() {
    if cfg!(debug_assertions)
        && let Some(game) = GAME_THREAD.get()
    {
        assert_eq!(
            *game,
            std::thread::current().id(),
            "a state of the game's thread was reached from another thread"
        );
    }
}

/// A static owner of state only the game's thread reaches (cells of `Plain` kinds, which are
/// `!Sync`).
///
/// The one `unsafe impl Sync` for them. The safety argument is the thread census
/// (`notes/w10_census.md`, section 1): the stores held here (service, records, saves,
/// install, ..) are read and written by the game's main thread alone; the audio, key,
/// network and drawing threads reach none of them. `get` checks it in debug builds
/// (`set_game_thread`). Anything a second thread must see stays in atomic cells, outside.
/// One exception, argued where it is made: a `DrawShared` store is held in a `GameThread`
/// and lent, read only, to the drawing threads for one drawing scope (`PhaseRead`).
pub struct GameThread<T>(T);

// SAFETY: reached only from the game's thread (above); `get` asserts it in debug builds.
unsafe impl<T> Sync for GameThread<T> {}

impl<T> GameThread<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// The state, from the game's thread.
    pub fn get(&self) -> &T {
        assert_game_thread();
        &self.0
    }

    /// The state from a thread other than the game's, for the two readers the original had
    /// there: the audio thread's file calls read the install texts' second base directory
    /// (`files::resolve_path`: atomic bytes, written only by the installer on the game's
    /// thread), and the network's receiving thread measures a chat line with the
    /// graphics owner's fonts (`net::add_chat_line`: read only, as the original's thread
    /// read them). Nothing else may use it: no other thread reaches a `GameThread`.
    pub(crate) fn on_any_thread(&self) -> &T {
        &self.0
    }
}

// ---- the drawing phase's stores (class D of `notes/m3_census.md`, section 2b) ----

/// A store of plain cells the game's thread writes between frames and every drawing part
/// reads while a frame's parts are drawn beside each other (`workers::draw_parts`): the
/// census' class D (the light tables, the course's and the frame's read groups, ..).
///
/// Outside a drawing scope it is a `GameThread` store: `get`, on the game's thread, is where
/// every write starts from. Inside one the parts (part 0 on the game's thread among them)
/// read it through the scope's `PhaseRead` only, handed to them by `workers::draw_parts`;
/// `get` there is a slip, and a debug build stops at it.
pub struct DrawShared<T>(GameThread<T>);

impl<T> DrawShared<T> {
    pub const fn new(value: T) -> Self {
        Self(GameThread::new(value))
    }

    /// The store, on the game's thread outside a drawing scope (debug builds check both).
    pub fn get(&self) -> &T {
        debug_assert!(
            !crate::workers::in_scope(),
            "a drawing store reached by its owner while the parts read it (take the part's reads)"
        );
        self.0.get()
    }

    /// The store's bytes, as the debug builds' lent ranges hold them.
    #[cfg(debug_assertions)]
    fn place(&self) -> (*const u8, usize) {
        (std::ptr::from_ref(&self.0.0).cast(), size_of::<T>())
    }
}

/// A `DrawShared` store lent, read only, to the parts of one drawing scope (`'s`): made only
/// by `workers::draw_parts` (`lend` takes the scope, which only it opens), it is shared by
/// reference with every part (`workers::Lent`) and reads the store as `&T`.
pub struct PhaseRead<'s, T>(&'s T);

impl<'s, T: crate::cells::Cells> PhaseRead<'s, T> {
    /// `store` lent to the parts while `scope` is open; debug builds note its bytes as lent
    /// (`plain_written` then stops at a write into them) until the scope ends.
    pub(crate) fn lend(store: &'s DrawShared<T>, scope: &'s crate::workers::DrawScope) -> Self {
        let _ = scope;
        #[cfg(debug_assertions)]
        {
            let (start, len) = store.place();
            lent::add(start, len);
        }
        // Not `get`: the scope is open (the store is read here, not written).
        Self(&store.0.0)
    }
}

impl<T> std::ops::Deref for PhaseRead<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

// SAFETY: sharing a `PhaseRead` between threads shares `&T` of a store of non-atomic cells
// (`T: !Sync`), which is sound when no thread writes the store while any thread may read
// it through the token. That holds for these reasons:
//
// 1. No writer inside the scope. A token exists only between `workers::DrawScope::open` and
//    the scope's end, both in `workers::draw_parts`: the parts' runs are spawned and joined
//    (rayon's `in_place_scope`, or one after another on the game's thread) within it, and
//    nothing else runs on the game's thread meanwhile, which is drawing part 0, a part like
//    the others. The parts write only their own state (`PartScratch`, their view's
//    `ViewScratch`, their own render cache) and never a class D store: the write census of
//    frame plan F7 and W10 M0b (`write_watch`, held races with one and two players and the
//    mirror beside its view), whose shared writes were removed or moved before the parts
//    (the scene clock per frame, the fog values, the dead stores). Every write to a class D
//    store happens on the game's thread through `DrawShared::get`, outside a scope. The
//    scope's start and end order them: the parts are started after the game's thread's last
//    write and joined before its next (rayon's spawn and join, or the same thread), so every
//    read sees the writes before it and none overlaps a later one.
// 2. The token cannot outlive the scope. It borrows the `DrawScope` (`'s`), which
//    `draw_parts` drops before anything runs after the parts (`after(0)`); and the parts are
//    handed only `&`-references made from it for the length of their call
//    (`workers::DrawReads`, a higher-ranked lifetime in `draw_parts`' part type), so a part
//    can keep none past its run. `PhaseRead` is neither `Clone` nor `Send`: it stays in the
//    `Lent` `draw_parts` holds.
// 3. `T: Cells` (records of the crate's cells): reading one is a plain load of each cell
//    read, never a write (no borrow counts, no lazy set-up), so the parts' reads at once do
//    not race each other.
//
// Checked in debug builds: `DrawShared::get` panics inside a scope or off the game's
// thread (a part reaching the store by its owner, or the game's thread writing it while
// drawing part 0); every plain cell's write (`plain_written`: `set`, `set_bits`, `update`,
// the byte cells, the cells' copies and merges) panics when the cell is in a lent store's
// bytes, whichever thread writes it. Release builds keep neither check and cost nothing.
//
// What would break it: a part (or code reached from one) writing a lent store, which the
// debug check catches at the write (the debug-build two-player race at each merge is the
// check); a store with a field another thread reads outside the scope (class S) made
// `DrawShared`; a part spawning work that outlives its run; a `get` taken before the scope
// and held into it (it is a plain `&T`: the scope's code takes none).
unsafe impl<T: crate::cells::Cells> Sync for PhaseRead<'_, T> {}

/// Debug builds: the byte ranges lent to the open drawing scope (`PhaseRead::lend`). One
/// writer (the game's thread, before the parts start and after they have all ended), many
/// readers: each part's writes look them up.
#[cfg(debug_assertions)]
mod lent {
    use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

    /// The most stores one scope lends.
    const MAX: usize = 64;
    static STARTS: [AtomicPtr<u8>; MAX] = [const { AtomicPtr::new(std::ptr::null_mut()) }; MAX];
    static ENDS: [AtomicPtr<u8>; MAX] = [const { AtomicPtr::new(std::ptr::null_mut()) }; MAX];
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    /// The `len` bytes from `start` lent.
    pub fn add(start: *const u8, len: usize) {
        let n = COUNT.load(Ordering::Relaxed);
        assert!(n < MAX, "more stores lent to a drawing scope than noted");
        STARTS[n].store(start.cast_mut(), Ordering::Relaxed);
        ENDS[n].store(start.wrapping_add(len).cast_mut(), Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Release);
    }

    pub fn clear() {
        COUNT.store(0, Ordering::Release);
    }

    /// Panics when the byte at `at` is lent.
    pub fn check(at: *const u8) {
        let n = COUNT.load(Ordering::Acquire);
        for k in 0..n {
            let (start, end) = (
                STARTS[k].load(Ordering::Relaxed).cast_const(),
                ENDS[k].load(Ordering::Relaxed).cast_const(),
            );
            assert!(
                !(start <= at && at < end),
                "a store lent to the drawing parts written while they read it"
            );
        }
    }
}

/// The drawing scope's end (`workers::DrawScope`): no store is lent any more.
pub(crate) fn end_lending() {
    #[cfg(debug_assertions)]
    lent::clear();
}

/// Debug builds: stops at a write into a store lent to the open drawing scope (`PhaseRead`).
/// Every plain cell's write calls it; release builds compile it to nothing.
#[inline(always)]
pub(crate) fn plain_written<T>(cell: &Cell<T>) {
    #[cfg(debug_assertions)]
    lent::check(cell.as_ptr().cast_const().cast());
    #[cfg(not(debug_assertions))]
    let _ = cell;
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
        // in its cells, `cells`); every byte of `value` is written before it is taken.
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

    /// The run of cells holding byte `offset`, at the width `WORDS` picks, and the byte it
    /// starts at: a 4-aligned record's whole dwords, or the bytes past them; a packed
    /// record's bytes.
    pub fn cell_run(&self, offset: usize) -> (usize, crate::cells::CellRun<'_>) {
        use crate::cells::CellRun;
        let size = size_of::<T>();
        let words = if Self::WORDS { size & !3 } else { 0 };
        let base = self.0.get().cast::<u8>();
        assert!(offset < size, "Shared: byte {offset} outside the record");
        if offset < words {
            // SAFETY: the record's whole dwords, 4-aligned (`WORDS`), reached only as these
            // `AtomicU32`s (`get`, `set` and the cells take the same width); they live as
            // long as `self`.
            (
                0,
                CellRun::Words(unsafe { std::slice::from_raw_parts(base.cast(), words / 4) }),
            )
        } else {
            // SAFETY: the bytes past them (a packed record: all of it), reached only as these
            // `AtomicU8`s, `u8`'s layout.
            let tail = unsafe { std::slice::from_raw_parts(base.add(words).cast(), size - words) };
            (words, CellRun::Bytes(tail))
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

/// A record's place by address, for the tests.
#[cfg(test)]
impl<T> Shared<T> {
    pub fn as_ptr(&self) -> *mut T {
        self.0.get()
    }
}

const fn zeros<const N: usize>() -> [Dword; N] {
    [const { Dword::new(0) }; N]
}

/// The screen: 0x4F494C .. 0x4F4984.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Screen {
    pub width: PlainDword,       // g_screenWidth
    pub height: PlainDword,      // g_screenHeight
    pub bits: PlainDword,        // g_screenBits
    pub bytes: PlainDword,       // g_screenBytesPerPixel
    pub row_bytes: PlainDword,   // g_screenRowBytes
    pub row_dwords: PlainDword,  // g_screenRowDwords
    pub depth_bits: PlainDword,  // Screen.depth_bits
    pub kind: PlainDword,        // Screen.kind
    pub format: [PlainDword; 6], // g_surfaceFormat (a pixel format record, 24 bytes)
}

impl Screen {
    pub const fn new() -> Self {
        Screen {
            width: PlainDword::new(0),
            height: PlainDword::new(0),
            bits: PlainDword::new(0),
            bytes: PlainDword::new(0),
            row_bytes: PlainDword::new(0),
            row_dwords: PlainDword::new(0),
            depth_bits: PlainDword::new(0),
            kind: PlainDword::new(0),
            format: [const { PlainDword::new(0) }; 6],
        }
    }
}

#[cfg(test)]
pub const SCREEN_LEN: usize = core::mem::size_of::<Screen>();

/// The window's state and the main loop's: 0x518370 .. 0x51839C.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WindowLoopState {
    pub f370: Dword,           // dword_518370
    pub window_active: Dword,  // g_windowActive
    pub window_focused: Dword, // g_windowFocused
    pub quitting: Flag,        // g_quitting
    pub lost: Dword,           // WindowLoopState.lost
    pub no_task_switch: Dword, // WindowLoopState.no_task_switch (`notaskswitch`, a `var\ral.cfg` setting)
    pub keep_priority: Dword,  // g_keepPriority (`noprichange`, a setting)
    /// `dword_51838C` and `g_mainLoopStack`: only the main loop's, which needs neither.
    pub _gap_51838c: [Dword; 2],
    pub f394: Dword,   // dword_518394 (1 at start)
    pub paused: Dword, // WindowLoopState.paused
}

impl WindowLoopState {
    pub const fn new() -> Self {
        WindowLoopState {
            f370: Dword::new(0),
            window_active: Dword::new(0),
            window_focused: Dword::new(0),
            quitting: Flag::new(false),
            lost: Dword::new(0),
            no_task_switch: Dword::new(0),
            keep_priority: Dword::new(0),
            _gap_51838c: [Dword::new(0), Dword::new(0)],
            f394: Dword::new(1),
            paused: Dword::new(0),
        }
    }
}

#[cfg(test)]
pub const FLAGS_LEN: usize = core::mem::size_of::<WindowLoopState>();

/// What the machine is and has, and the window handles: 0x5183A8 .. 0x5184A8.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Machine {
    pub has_mmx: Dword,          // g_hasMmx
    pub ignore_mmx: Dword,       // Machine.ignore_mmx (`ignoremmx`)
    pub has_sse: Dword,          // g_hasSse
    pub ignore_katmai: Dword,    // Machine.ignore_katmai (`ignorekatmai`)
    pub has_3dnow: Dword,        // g_has3dNow
    pub ignore_3dnow: Dword,     // Machine.ignore_3dnow (`ignoreamdk3d`)
    pub work: Dword,             // Machine.work
    pub work_float: Dword,       // Machine.work_float
    pub gap_c8: Dword,           // 0x5183C8
    pub scratch: Dword,          // g_soundScratch: a scratch dword
    pub cpu_family: Dword,       // g_cpuFamily
    pub cpu_signature: Dword,    // g_cpuSignature
    pub cpu_vendor: [Dword; 3],  // g_cpuVendor: the vendor's text
    pub gap_e4: Dword,           // 0x5183E4
    pub cpu_mhz: Dword,          // g_cpuMhz
    pub has_fpu: Dword,          // g_hasFpu
    pub has_tsc: Dword,          // g_hasTsc
    pub has_cmov: Dword,         // g_hasCmov
    pub found_mmx: Dword,        // Machine.found_mmx
    pub found_sse: Dword,        // Machine.found_sse
    pub found_3dnow: Dword,      // Machine.found_3dnow
    pub keyboard_type: Dword,    // g_keyboardType (1 at start)
    pub version: [Dword; 37],    // VersionInformation (OSVERSIONINFOA, 148 bytes)
    pub exception_resume: Dword, // g_exceptionResume
    pub instance: Dword,         // hInstance
    pub window: Dword,           // hWnd
}

impl Machine {
    pub const fn new() -> Self {
        Machine {
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
        }
    }
}

#[cfg(test)]
pub const MACHINE_LEN: usize = core::mem::size_of::<Machine>();

/// The key thread's count and the clocks the menus keep: 0x518998 .. 0x5189A4.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Clocks {
    pub key_ticks: Dword,  // Clocks.key_ticks: the key thread's turns
    pub frame: Dword,      // Clocks.frame: the menus' clock (`update_frame_clock`)
    pub setup_time: Dword, // Clocks.setup_time
}

impl Clocks {
    pub const fn new() -> Self {
        Clocks {
            key_ticks: Dword::new(0),
            frame: Dword::new(0),
            setup_time: Dword::new(0),
        }
    }
}

#[cfg(test)]
pub const CLOCKS_LEN: usize = core::mem::size_of::<Clocks>();

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
/// loader swaps): a typed buffer (`Bytes<N>`) or a place in one.
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

/// What a buffer of the game's bytes offers whether it is shared (`Bytes`) or the game
/// thread's (`PlainBytes`): the code that reads or writes a typed buffer without caring which.
pub trait ByteBuffer {
    fn read_vec(&self) -> Vec<u8>;
    fn get_u32_at(&self, offset: usize) -> u32;
    fn get_byte(&self, index: usize) -> u8;
    fn read_bytes_at(
        &self,
        offset: usize,
        len: usize,
        what: &str,
    ) -> Result<Vec<u8>, crate::errors::BadInput>;
    fn write_bytes_at(
        &self,
        offset: usize,
        data: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput>;
}

impl<const N: usize> ByteBuffer for Bytes<N> {
    fn read_vec(&self) -> Vec<u8> {
        self.read().to_vec()
    }
    fn get_byte(&self, index: usize) -> u8 {
        self.get(index)
    }
    fn get_u32_at(&self, offset: usize) -> u32 {
        self.get_u32(offset)
    }
    fn read_bytes_at(
        &self,
        offset: usize,
        len: usize,
        what: &str,
    ) -> Result<Vec<u8>, crate::errors::BadInput> {
        self.read_at(offset, len, what)
    }
    fn write_bytes_at(
        &self,
        offset: usize,
        data: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        self.write_at(offset, data, what)
    }
}

impl<const N: usize> ByteBuffer for PlainBytes<N> {
    fn read_vec(&self) -> Vec<u8> {
        self.read().to_vec()
    }
    fn get_byte(&self, index: usize) -> u8 {
        self.get(index)
    }
    fn get_u32_at(&self, offset: usize) -> u32 {
        self.get_u32(offset)
    }
    fn read_bytes_at(
        &self,
        offset: usize,
        len: usize,
        what: &str,
    ) -> Result<Vec<u8>, crate::errors::BadInput> {
        self.read_at(offset, len, what)
    }
    fn write_bytes_at(
        &self,
        offset: usize,
        data: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        self.write_at(offset, data, what)
    }
}

impl<const N: usize> TextPlace for PlainBytes<N> {
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

/// A text in a buffer of bytes that begins `from` bytes in (a path among several in one
/// buffer): read up to its NUL inside the buffer, written to inside it.
pub struct TextFrom<'a, B: ByteBuffer> {
    buffer: &'a B,
    from: usize,
}

impl<const N: usize> PlainBytes<N> {
    /// The text that begins `from` bytes in.
    pub fn text_from(&self, from: usize) -> TextFrom<'_, Self> {
        TextFrom { buffer: self, from }
    }
}

impl<B: ByteBuffer> TextPlace for TextFrom<'_, B> {
    fn text(&self, _what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        let all = self.buffer.read_vec();
        let rest = all
            .get(self.from..)
            .ok_or(crate::errors::BadInput("a text past its buffer"))?;
        match rest.iter().position(|&c| c == 0) {
            Some(n) => Ok(rest[..n].to_vec()),
            None => Err(crate::errors::BadInput("a text with no end in its buffer")),
        }
    }

    fn write_text_at(
        &self,
        at: usize,
        bytes: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        self.buffer
            .write_bytes_at(self.from.saturating_add(at), bytes, what)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::first_values::assert_block_starts_as_the_games;

    #[test]
    fn the_blocks_start_as_the_games() {
        let system = system();
        assert_block_starts_as_the_games(system.session.screen.get());
        assert_block_starts_as_the_games(&system.session.flags);
        assert_block_starts_as_the_games(&system.session.machine);
        assert_block_starts_as_the_games(&system.session.clocks);
    }

    #[test]
    fn the_blocks_keep_the_images_layout() {
        use core::mem::offset_of;
        assert_eq!(SCREEN_LEN, 0x38);
        assert_eq!(offset_of!(Screen, format), 0x20);
        assert_eq!(FLAGS_LEN, 0x2C);
        assert_eq!(offset_of!(WindowLoopState, quitting), 0xC);
        assert_eq!(offset_of!(WindowLoopState, paused), 0x28);
        assert_eq!(MACHINE_LEN, 0x100);
        assert_eq!(offset_of!(Machine, scratch), 0x24);
        assert_eq!(offset_of!(Machine, cpu_vendor), 0x30);
        assert_eq!(offset_of!(Machine, cpu_mhz), 0x40);
        assert_eq!(offset_of!(Machine, found_3dnow), 0x58);
        assert_eq!(offset_of!(Machine, version), 0x60);
        assert_eq!(offset_of!(Machine, exception_resume), 0xF4);
        assert_eq!(offset_of!(Machine, window), 0xFC);
        assert_eq!(CLOCKS_LEN, 12);
    }

    /// A debug build stops at a plain cell's write into a store lent to the drawing parts,
    /// and only there: the cell next to it is written, the lent one keeps its value.
    #[test]
    #[cfg(debug_assertions)]
    fn a_write_into_a_lent_store_stops_a_debug_build() {
        let _g = crate::test_support::test_lock();
        let cells = [PlainDword::new(1), PlainDword::new(2)];
        lent::add(std::ptr::from_ref(&cells[1]).cast(), 4);
        cells[0].set(3);
        let hit = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cells[1].set(4)));
        let copied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::cells::Cells::copy_from(&cells[1], &cells[0]);
        }));
        end_lending();
        assert!(hit.is_err() && copied.is_err());
        assert_eq!([cells[0].get(), cells[1].get()], [3, 2]);
        cells[1].set(5);
        assert_eq!(cells[1].get(), 5);
    }
}
