//! The session core: the variables many modules and the game's threads share. The screen's size
//! and format, the window's state, what the processor is and has, the window handles, and the
//! clocks the key thread and the menus keep.
//!
//! Each dword is an atomic (`Dword`, plain loads and stores: the audio, key and network threads
//! read them without a lock).

#![allow(unsafe_code)]

use std::cell::{Cell, UnsafeCell};
use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, Ordering::Relaxed};

/// One of the game's dwords: an atomic cell the game's threads read and write.
#[repr(transparent)]
pub struct Dword(AtomicU32);

impl Dword {
    pub const fn new(value: u32) -> Self {
        Self(AtomicU32::new(value))
    }

    /// A row of cells holding `values`, in order (a store's starting table).
    pub const fn array<const N: usize>(values: [u32; N]) -> [Self; N] {
        let mut cells: [Self; N] = [const { Self::new(0) }; N];
        let mut i = 0;
        while i < N {
            cells[i] = Self::new(values[i]);
            i += 1;
        }
        cells
    }

    pub fn get(&self) -> u32 {
        self.0.load(Relaxed)
    }

    pub fn set(&self, value: u32) {
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

/// The bits of a run of dwords or floats (`Cell32`), as the bytes read whole.
pub fn read_bits<C: Cell32, const N: usize>(cells: &[C; N]) -> [u32; N] {
    std::array::from_fn(|k| cells[k].bits())
}

/// Stores a run of dwords or floats by their bits.
pub fn write_bits<C: Cell32, const N: usize>(cells: &[C; N], values: [u32; N]) {
    for (cell, value) in cells.iter().zip(values) {
        cell.set_bits(value);
    }
}

/// The bits of the first `N` cells of a longer run (part of a table read whole).
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

    /// From the float's bits (the starting values are given as bits).
    pub const fn from_bits(bits: u32) -> Self {
        Self(AtomicU32::new(bits))
    }

    pub fn get(&self) -> f32 {
        f32::from_bits(self.0.load(Relaxed))
    }

    pub fn set(&self, value: f32) {
        self.0.store(value.to_bits(), Relaxed);
    }

    /// The atomic holding its bits (the cells' copies, `cells::CellRef`).
    pub fn atomic(&self) -> &AtomicU32 {
        &self.0
    }

    /// The cell's 32 bits (read by `x87_float::ld`).
    pub fn bits(&self) -> u32 {
        self.0.load(Relaxed)
    }

    /// Stores the 32 bits (written by `x87_float::st`).
    pub fn set_bits(&self, bits: u32) {
        self.0.store(bits, Relaxed);
    }
}

/// A signed 32-bit value other threads reach: the same four bytes as a `Dword`, read and written
/// as `i32` (an atomic cell).
#[repr(transparent)]
pub struct Int(AtomicU32);

impl Int {
    pub const fn new(value: i32) -> Self {
        Self(AtomicU32::new(value as u32))
    }

    pub fn get(&self) -> i32 {
        self.0.load(Relaxed) as i32
    }

    pub fn set(&self, value: i32) {
        self.0.store(value as u32, Relaxed);
    }

    /// The atomic holding the bits (the cells' copies, `cells::CellRef`).
    pub fn atomic(&self) -> &AtomicU32 {
        &self.0
    }
}

/// Cell `n` of `cells`. Past the table's end it gives what follows, but only inside `holder`, the
/// store or record holding the table; beyond that the next thing in memory is another static, so
/// that is refused.
#[cfg(test)]
pub fn read_on_in<H: crate::memory::cells::ByteView, C: Cell32>(
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
                crate::memory::cells::load_cells(holder, at, &mut bytes)
            })?;
            Ok(u32::from_le_bytes(bytes))
        }
    }
}

/// `read_on_in`'s write.
#[cfg(test)]
pub fn write_on_in<H: crate::memory::cells::ByteView, C: Cell32>(
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
            crate::memory::cells::store_cells(holder, at, &v.to_le_bytes())
        }),
    }
}

/// `access` on dword `n` of `cells`, placed from `holder`'s first byte (refused, and logged,
/// when that is not inside `holder`).
#[cfg(test)]
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
        crate::memory::host_hooks::log(&format!(
            "pointers: {what}: dword {n} at +{at:#x}, outside its holder of {:#x} bytes",
            size_of::<H>()
        ));
    })
}

/// A run of 32-bit values in a record: one cell, an array of runs, or the dwords of a buffer of
/// bytes (a gap).
pub trait Run {
    fn count(&self) -> usize;
    fn bits(&self, i: usize) -> u32;
    fn set_bits(&self, i: usize, v: u32);
}

impl<T: Cell32> Run for T {
    fn count(&self) -> usize {
        1
    }
    fn bits(&self, _: usize) -> u32 {
        Cell32::bits(self)
    }
    fn set_bits(&self, _: usize, v: u32) {
        Cell32::set_bits(self, v);
    }
}

/// An array of runs of one type, each the same size.
impl<R: Run, const N: usize> Run for [R; N] {
    fn count(&self) -> usize {
        self.first().map_or(0, |each| N * each.count())
    }
    fn bits(&self, i: usize) -> u32 {
        let each = self[0].count();
        self[i / each].bits(i % each)
    }
    fn set_bits(&self, i: usize, v: u32) {
        let each = self[0].count();
        self[i / each].set_bits(i % each, v);
    }
}

/// The whole dwords of a buffer (a gap between a record's fields).
impl<const N: usize> Run for PlainBytes<N> {
    fn count(&self) -> usize {
        N / 4
    }
    fn bits(&self, i: usize) -> u32 {
        self.get_u32(4 * i)
    }
    fn set_bits(&self, i: usize, v: u32) {
        self.set_u32(4 * i, v);
    }
}

/// The whole dwords of an atomic buffer (a record's filler).
impl<const N: usize> Run for Bytes<N> {
    fn count(&self) -> usize {
        N / 4
    }
    fn bits(&self, i: usize) -> u32 {
        self.get_u32(4 * i)
    }
    fn set_bits(&self, i: usize, v: u32) {
        self.set_u32(4 * i, v);
    }
}

/// Where dword `n` of `runs` laid end to end is: a table and the fields after it in the record,
/// named in the record's order. A table read by an index past its end gives the next field's
/// dword. Past the last run is refused, and logged.
fn locate<'a>(
    runs: &[&'a dyn Run],
    n: usize,
    what: &str,
) -> Result<(&'a dyn Run, usize), crate::errors::BadInput> {
    let mut rest = n;
    for run in runs {
        if rest < run.count() {
            return Ok((*run, rest));
        }
        rest -= run.count();
    }
    crate::memory::host_hooks::log(&format!(
        "pointers: {what}: dword {n}, past the {} dwords listed",
        n - rest
    ));
    Err(crate::errors::BadInput(
        "a dword past the fields of its record",
    ))
}

/// Dword `n` of `runs` (see `locate`).
pub fn read_runs(runs: &[&dyn Run], n: usize, what: &str) -> Result<u32, crate::errors::BadInput> {
    let (run, i) = locate(runs, n, what)?;
    Ok(run.bits(i))
}

/// The bytes of `runs` laid end to end, from byte `at`, into `to` (little-endian within each
/// dword, as the game's records are read). A byte past the last run is refused, and logged.
pub fn read_run_bytes(
    runs: &[&dyn Run],
    at: usize,
    to: &mut [u8],
    what: &str,
) -> Result<(), crate::errors::BadInput> {
    for (k, out) in to.iter_mut().enumerate() {
        let byte = at.checked_add(k).ok_or(crate::errors::BadInput(
            "a byte past the fields of its record",
        ))?;
        let (run, i) = locate(runs, byte / 4, what)?;
        *out = run.bits(i).to_le_bytes()[byte % 4];
    }
    Ok(())
}

/// `read_runs`' write.
pub fn write_runs(
    runs: &[&dyn Run],
    n: usize,
    v: u32,
    what: &str,
) -> Result<(), crate::errors::BadInput> {
    let (run, i) = locate(runs, n, what)?;
    run.set_bits(i, v);
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
        self.0.load(Relaxed) != 0
    }

    pub fn set(&self, value: bool) {
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
        self.0.load(Relaxed)
    }

    pub fn set(&self, value: u16) {
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

/// A buffer of the game's bytes (a table, a record handed on by address), shared between the game's
/// threads as plain bytes are: each byte an atomic of its own, reached at that width only
/// (`memory::shared_memory` copies byte by byte too), so no access mixes sizes.
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
        self.0[index].load(Relaxed)
    }

    pub fn set(&self, index: usize, value: u8) {
        self.0[index].store(value, Relaxed);
    }

    /// The byte at `index` as an atomic.
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

    /// The byte at `offset`, as `read_at(offset, 1, what)` gives it, without the allocation.
    pub fn byte_at(&self, offset: usize, what: &str) -> Result<u8, crate::errors::BadInput> {
        self.range(offset, 1, what)?;
        Ok(self.get(offset))
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
                crate::memory::host_hooks::log(&format!(
                    "session: {what}: {len} bytes at {offset:#x} of a {N:#x}-byte buffer"
                ));
                Err(crate::errors::BadInput("past the end of a buffer"))
            }
        }
    }
}

/// A dword at an address that is not a multiple of 4 (the audio block's has-3D flag): four
/// byte atomics, as `Bytes`.
#[repr(transparent)]
pub(super) struct Loose([AtomicU8; 4]);

impl Loose {
    pub const fn new(value: u32) -> Self {
        Self(atomic_bytes(value.to_le_bytes()))
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
#[repr(transparent)]
pub struct Plain<T: Copy>(Cell<T>);

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

impl PlainDword {
    pub const fn new(value: u32) -> Self {
        Self(Cell::new(value))
    }

    /// A row of cells holding `values`, in order (a store's starting table).
    pub const fn array<const N: usize>(values: [u32; N]) -> [Self; N] {
        let mut cells: [Self; N] = [const { Self::new(0) }; N];
        let mut i = 0;
        while i < N {
            cells[i] = Self::new(values[i]);
            i += 1;
        }
        cells
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

impl PlainFloat {
    /// A cell holding `value`.
    pub const fn new(value: f32) -> Self {
        Self(Cell::new(value.to_bits()))
    }

    /// A row of cells holding `values`, in order (a store's starting table).
    pub const fn array<const N: usize>(values: [f32; N]) -> [Self; N] {
        let mut cells: [Self; N] = [const { Self::new(0.0) }; N];
        let mut i = 0;
        while i < N {
            cells[i] = Self::new(values[i]);
            i += 1;
        }
        cells
    }

    /// A row of cells holding the floats whose bits are `values`, in order.
    pub const fn bits_array<const N: usize>(values: [u32; N]) -> [Self; N] {
        let mut cells: [Self; N] = [const { Self::new(0.0) }; N];
        let mut i = 0;
        while i < N {
            cells[i] = Self::from_bits(values[i]);
            i += 1;
        }
        cells
    }

    /// From the float's bits (the starting values are given as bits).
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

/// A signed 32-bit value only the game's thread reaches: the same four bytes as a dword,
/// read and written as `i32`.
#[repr(transparent)]
pub struct PlainInt(Cell<u32>);

impl PlainInt {
    pub const fn new(value: i32) -> Self {
        Self(Cell::new(value as u32))
    }

    pub fn get(&self) -> i32 {
        self.0.get() as i32
    }

    pub fn set(&self, value: i32) {
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

impl Cell32 for Int {
    fn bits(&self) -> u32 {
        self.0.load(Relaxed)
    }
    fn set_bits(&self, bits: u32) {
        self.0.store(bits, Relaxed);
    }
}

impl Cell32 for PlainInt {
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

    /// The byte at `offset`, as `read_at(offset, 1, what)` gives it, without the allocation.
    pub fn byte_at(&self, offset: usize, what: &str) -> Result<u8, crate::errors::BadInput> {
        self.range(offset, 1, what)?;
        Ok(self.get(offset))
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
                crate::memory::host_hooks::log(&format!(
                    "session: {what}: {len} bytes at {offset:#x} of a {N:#x}-byte buffer"
                ));
                Err(crate::errors::BadInput("past the end of a buffer"))
            }
        }
    }
}

/// `value`'s bytes as cells (a `const` initialiser of `PlainBytes`), in one
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

/// A signed cell read and written as a number, shared (`Int`) or the game thread's
/// (`PlainInt`); its bits through `Cell32`.
pub trait IntLike: Cell32 {
    fn get(&self) -> i32;
    fn set(&self, value: i32);
}

impl IntLike for Int {
    fn get(&self) -> i32 {
        Int::get(self)
    }
    fn set(&self, value: i32) {
        Int::set(self, value);
    }
}

impl IntLike for PlainInt {
    fn get(&self) -> i32 {
        PlainInt::get(self)
    }
    fn set(&self, value: i32) {
        PlainInt::set(self, value);
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

/// The kind of cells a record is made of. Only the plain cells (`PlainCells`) are used now;
/// the records are written over a kind `K` (as `Car<K>`, `Vertex<K>`) from the time a view's
/// copy of its car was atomic, and could be written over plain cells alone.
pub trait CellKind {
    type Dword: crate::memory::cells::Cells + DwordLike + Cell32;
    type Float: crate::memory::cells::Cells + FloatLike;
    type Int: crate::memory::cells::Cells + IntLike;
    type Bytes<const N: usize>: crate::memory::cells::Cells;
}

/// The game thread's cells (`PlainDword`, `PlainFloat`, `PlainBytes`).
pub struct PlainCells;

impl CellKind for PlainCells {
    type Dword = PlainDword;
    type Float = PlainFloat;
    type Int = PlainInt;
    type Bytes<const N: usize> = PlainBytes<N>;
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
/// The one `unsafe impl Sync` for them. The safety argument is the thread census (an audit of which
/// thread reaches which store): the stores held here (service, records, saves, install, ..) are
/// read and written by the game's main thread alone; the audio, key, network and drawing threads
/// reach none of them. `get` checks it in debug builds (`set_game_thread`). Anything a second
/// thread must see stays in atomic cells, outside. One exception, argued where it is made: a
/// `DrawShared` store is held in a `GameThread` and lent, read only, to the drawing threads for one
/// drawing scope (`PhaseRead`).
#[repr(transparent)]
pub struct GameThread<T>(T);

// SAFETY: reached only from the game's thread (above); `get` asserts it in debug builds.
unsafe impl<T> Sync for GameThread<T> {}

/// A record of the game's thread held among records of shared cells reads through the
/// wrapper as through `get` (`state.results.finished`).
impl<T> core::ops::Deref for GameThread<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> GameThread<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// The state, from the game's thread.
    pub fn get(&self) -> &T {
        assert_game_thread();
        &self.0
    }
}

// ---- the drawing phase's stores ----

/// A store of plain cells the game's thread writes between frames and every drawing part reads
/// while a frame's parts are drawn beside each other (`scene::workers::draw_parts`): the light tables, the
/// course's and the frame's read groups, ..
///
/// Outside a drawing scope it is a `GameThread` store: `get`, on the game's thread, is where
/// every write starts from. Inside one the parts (part 0 on the game's thread among them)
/// read it through the scope's `PhaseRead` only, handed to them by `scene::workers::draw_parts`;
/// `get` there is a slip, and a debug build stops at it.
pub struct DrawShared<T>(GameThread<T>);

impl<T> DrawShared<T> {
    pub const fn new(value: T) -> Self {
        Self(GameThread::new(value))
    }

    /// The store, on the game's thread outside a drawing scope (debug builds check both).
    pub fn get(&self) -> &T {
        debug_assert!(
            !crate::memory::draw_order::in_scope(),
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
/// by `scene::workers::draw_parts` (`lend` takes the scope, which only it opens), it is shared by
/// reference with every part (`scene::workers::Lent`) and reads the store as `&T`.
pub struct PhaseRead<'s, T>(&'s T);

impl<'s, T: crate::memory::cells::Cells> PhaseRead<'s, T> {
    /// `store` lent to the parts while `scope` is open; debug builds note its bytes as lent
    /// (`plain_written` then stops at a write into them) until the scope ends.
    pub(crate) fn lend(
        store: &'s DrawShared<T>,
        scope: &'s crate::memory::draw_order::DrawScope,
    ) -> Self {
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

// SAFETY: sharing a `PhaseRead` between threads shares `&T` of a store of non-atomic cells (`T:
// !Sync`), which is sound when no thread writes the store while any thread may read it through the
// token. That holds for these reasons:
//
// 1. No writer inside the scope. A token exists only between `scene::workers::DrawScope::open` and the
//    scope's end, both in `scene::workers::draw_parts`: the parts' runs are spawned and joined (rayon's
//    `in_place_scope`, or one after another on the game's thread) within it, and nothing else runs
//    on the game's thread meanwhile, which is drawing part 0, a part like the others. The parts
//    write only their own state (`PartScratch`, their view's `ViewScratch`, their own render
//    cache) and never a draw-shared store: an audit build run on held races with one and two
//    players and the mirror beside its view found the shared writes, which were removed or moved
//    before the parts (the scene clock per frame, the fog values, the dead stores). Every write to
//    a draw-shared store happens on the game's thread through `DrawShared::get`, outside a scope.
//    The scope's start and end order them: the parts are started after the game's thread's last
//    write and joined before its next (rayon's spawn and join, or the same thread), so every read
//    sees the writes before it and none overlaps a later one.
// 2. The token cannot outlive the scope. It borrows the `DrawScope` (`'s`), which `draw_parts`
//    drops before anything runs after the parts (`after(0)`); and the parts are handed only
//    `&`-references made from it for the length of their call (`scene::workers::DrawReads`, a
//    higher-ranked lifetime in `draw_parts`' part type), so a part can keep none past its run.
//    `PhaseRead` is neither `Clone` nor `Send`: it stays in the `Lent` `draw_parts` holds. The one
//    part handed the `Lent` itself (part 0's `scene::workers::Helper`, for a piece of its drawing on
//    another thread, `scene::workers::draw_aside`) uses it only inside `draw_aside`, which joins the
//    piece (rayon's `in_place_scope`) before it returns, inside the part's call.
// 3. `T: Cells` (records of the crate's cells): reading one is a plain load of each cell read,
//    never a write (no borrow counts, no lazy set-up), so the parts' reads at once do not race
//    each other.
//
// Checked in debug builds: `DrawShared::get` panics inside a scope or off the game's thread (a
// part reaching the store by its owner, or the game's thread writing it while drawing part 0);
// every plain cell's write (`plain_written`: `set`, `set_bits`, `update`, the byte cells, the
// cells' copies and merges) panics when the cell is in a lent store's bytes, whichever thread
// writes it. Release builds keep neither check and cost nothing.
//
// What would break it: a part (or code reached from one) writing a lent store, which the debug
// check catches at the write (the debug-build two-player race at each merge is the check); a store
// with a field another thread reads outside the scope made `DrawShared`; a part spawning work that
// outlives its run; a `get` taken before the scope and held into it (it is a plain `&T`: the
// scope's code takes none).
unsafe impl<T: crate::memory::cells::Cells> Sync for PhaseRead<'_, T> {}

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

/// The drawing scope's end (`scene::workers::DrawScope`): no store is lent any more.
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

/// A record the game keeps as a plain struct (a voice, a mixer channel, a stream player, a list of
/// memory blocks, a caps record): copied out and in whole, or borrowed for the code that works on
/// it as the core's struct. The audio thread's turn and the game thread's share it by raw access.
#[repr(transparent)]
pub struct Shared<T>(UnsafeCell<T>);

// SAFETY: the record's bytes are reached only by `get`, `set` and `cell_run` (`as_ptr` is the
// tests'), each through relaxed atomics at the one width `WORDS` picks for each byte
// (`memory::shared_memory::load_words` / `store_words`, or byte by byte), so threads sharing it meet in
// atomic accesses only, never a data race or a mixed-size one; `T: Plain` (their bound), so a
// record read half written by another thread is still a valid `T`.
unsafe impl<T> Sync for Shared<T> {}

impl<T: crate::memory::cells::Plain> Shared<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    /// Whether the record is reached by dwords (4-aligned: each whole dword one atomic) or
    /// byte by byte (a packed record).
    pub const WORDS: bool = align_of::<T>() >= 4;

    /// A copy of the record (a record's fields may be read half changed; a 4-aligned record's
    /// dwords are each read whole).
    pub fn get(&self) -> T {
        let mut value = std::mem::MaybeUninit::<T>::uninit();
        // SAFETY: the record's own `size_of::<T>()` bytes, 4-aligned when `WORDS` (its
        // alignment), reached only at the widths `WORDS` picks (here, in `set` and in its cells,
        // `cell_run`); `to` is `value`'s private bytes. Every byte of `value` is written before
        // it is taken, and any bytes make a `T` (`Plain`), torn or not.
        unsafe {
            let to =
                std::slice::from_raw_parts_mut(value.as_mut_ptr().cast::<u8>(), size_of::<T>());
            let base = self.0.get().cast::<u8>();
            if Self::WORDS {
                crate::memory::shared_memory::load_words(base, size_of::<T>(), 0, to);
            } else {
                crate::memory::shared_memory::load(base, to.as_mut_ptr(), to.len());
            }
            value.assume_init()
        }
    }

    /// The run of cells holding byte `offset`, at the width `WORDS` picks, and the byte it
    /// starts at: a 4-aligned record's whole dwords, or the bytes past them; a packed
    /// record's bytes.
    pub fn cell_run(&self, offset: usize) -> (usize, crate::memory::cells::CellRun<'_>) {
        use crate::memory::cells::CellRun;
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
                crate::memory::shared_memory::store_words(base, size_of::<T>(), 0, from);
            } else {
                crate::memory::shared_memory::store(base, from.as_ptr(), from.len());
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

/// The screen's size and pixel format.
#[derive(rc2k_derive::Cells)]
pub struct Screen {
    pub width: PlainDword,
    pub height: PlainDword,
    pub bits: PlainDword,
    pub bytes: PlainDword,
    pub row_bytes: PlainDword,
    pub row_dwords: PlainDword,
    pub depth_bits: PlainDword,
    pub kind: PlainDword,
    pub format: [PlainDword; 6], // a pixel format record, 24 bytes
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

/// The window's flags the other threads read or write: `quitting` (the audio thread, the key
/// thread, the network's receiving thread, the file loads), `window_focused` (the key thread,
/// the SDL audio callback) and `window_active` (the audio thread's restore). Atomic cells;
/// the rest of the main loop's state is `WindowLoopState`, the game's thread alone.
#[derive(rc2k_derive::Cells)]
pub struct WindowShared {
    pub window_active: Dword,
    pub window_focused: Dword,
    pub quitting: Flag,
}

impl WindowShared {
    pub const fn new() -> Self {
        WindowShared {
            window_active: Dword::new(0),
            window_focused: Dword::new(0),
            quitting: Flag::new(false),
        }
    }
}

/// The window's state and the main loop's that only the game's thread reaches, less the three
/// flags of `WindowShared`.
#[derive(rc2k_derive::Cells)]
pub struct WindowLoopState {
    pub f370: PlainDword,
    pub lost: PlainDword,
    pub no_task_switch: PlainDword, // `notaskswitch`, a `var\ral.cfg` setting
    /// Only the main loop's, which needs neither.
    pub _gap_51838c: [PlainDword; 2],
    pub f394: PlainDword, // 1 at start
    pub paused: PlainDword,
}

impl WindowLoopState {
    pub const fn new() -> Self {
        WindowLoopState {
            f370: PlainDword::new(0),
            lost: PlainDword::new(0),
            no_task_switch: PlainDword::new(0),
            _gap_51838c: [PlainDword::new(0), PlainDword::new(0)],
            f394: PlainDword::new(1),
            paused: PlainDword::new(0),
        }
    }
}

/// The two machine facts the audio thread reads: the window handle (`hWnd`, the sound device's
/// cooperative level) and the FPU flag (the listener's set-up). Atomic cells; the rest is
/// `Machine`, the game's thread alone.
#[derive(rc2k_derive::Cells)]
pub struct HostShared {
    pub has_fpu: Dword,
    pub window: Dword,
}

impl HostShared {
    pub const fn new() -> Self {
        HostShared {
            has_fpu: Dword::new(0),
            window: Dword::new(0),
        }
    }
}

/// What the machine is and has, and the window handles that only the game's thread reaches:
/// less `HostShared`'s two.
#[derive(rc2k_derive::Cells)]
pub struct Machine {
    pub work: PlainDword,
    pub work_float: PlainDword,
    pub gap_c8: PlainDword,
    pub scratch: PlainDword, // a scratch dword
    pub cpu_family: PlainDword,
    pub cpu_signature: PlainDword,
    pub cpu_vendor: [PlainDword; 3], // the vendor's text
    pub gap_e4: PlainDword,
    pub cpu_mhz: PlainDword,
    pub has_tsc: PlainDword,
    pub has_cmov: PlainDword,
    pub found_mmx: PlainDword,
    pub found_sse: PlainDword,
    pub found_3dnow: PlainDword,
    pub keyboard_type: PlainDword, // 1 at start
    pub version: [PlainDword; 37], // OSVERSIONINFOA, 148 bytes
    pub exception_resume: PlainDword,
    pub instance: PlainDword,
    /// The setting `nofmv`: 1 = no films.
    pub no_fmv: PlainDword,
}

impl Machine {
    pub const fn new() -> Self {
        Machine {
            work: PlainDword::new(0),
            work_float: PlainDword::new(0),
            gap_c8: PlainDword::new(0),
            scratch: PlainDword::new(0),
            cpu_family: PlainDword::new(0),
            cpu_signature: PlainDword::new(0),
            cpu_vendor: [const { PlainDword::new(0) }; 3],
            gap_e4: PlainDword::new(0),
            cpu_mhz: PlainDword::new(0),
            has_tsc: PlainDword::new(0),
            has_cmov: PlainDword::new(0),
            found_mmx: PlainDword::new(0),
            found_sse: PlainDword::new(0),
            found_3dnow: PlainDword::new(0),
            keyboard_type: PlainDword::new(1),
            version: [const { PlainDword::new(0) }; 37],
            exception_resume: PlainDword::new(0),
            instance: PlainDword::new(0),
            no_fmv: PlainDword::new(0),
        }
    }
}

/// The key thread's count of turns (`Clocks.key_ticks`), which the game's thread reads for
/// its waits and the lists' clocks. One atomic cell; the menus' clocks are `Clocks`.
#[derive(rc2k_derive::Cells)]
pub struct KeyClock {
    pub ticks: Dword,
}

impl KeyClock {
    pub const fn new() -> Self {
        KeyClock {
            ticks: Dword::new(0),
        }
    }
}

/// The clocks the menus keep (the game's thread alone), less the key thread's count (`KeyClock`).
#[derive(rc2k_derive::Cells)]
pub struct Clocks {
    pub frame: PlainDword, // the menus' clock (`update_frame_clock`)
    pub setup_time: PlainDword,
}

impl Clocks {
    pub const fn new() -> Self {
        Clocks {
            frame: PlainDword::new(0),
            setup_time: PlainDword::new(0),
        }
    }
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

/// A text of the program's own (a literal with its NUL): read only. Writes into it (a missing
/// `.png`'s ending swapped) are refused.
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

// The lending checks exist only in debug builds.
#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;

    /// A debug build stops at a plain cell's write into a store lent to the drawing parts,
    /// and only there: the cell next to it is written, the lent one keeps its value.
    #[test]
    fn a_write_into_a_lent_store_stops_a_debug_build() {
        let _g = crate::test_support::test_lock();
        let cells = [PlainDword::new(1), PlainDword::new(2)];
        lent::add(std::ptr::from_ref(&cells[1]).cast(), 4);
        cells[0].set(3);
        let hit = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cells[1].set(4)));
        let copied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::memory::cells::Cells::copy_from(&cells[1], &cells[0]);
        }));
        end_lending();
        assert!(hit.is_err() && copied.is_err());
        assert_eq!([cells[0].get(), cells[1].get()], [3, 2]);
        cells[1].set(5);
        assert_eq!(cells[1].get(), 5);
    }
}
