//! The cells the game's data lives in: `Cells` (a record of atomics and cells whose runs are
//! found by byte offset), its runs and references, `Plain` values, `Store` (what a byte offset
//! resolves to), and the copies between them. Re-exported by `pointers` (the address layer
//! the conversion removes), so callers keep their paths until they are rewritten.

#![allow(unsafe_code)]

use crate::errors::BadInput;
use crate::framework::log;
use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, Ordering::Relaxed};

/// A type made of atomics and cells only (`session::{Dword, Float, Word, Byte, Bytes<N>}`, their
/// arrays, `#[repr(C)]` records of them), which `GamePtr::record` may hand out shared.
///
/// # Safety
/// Every byte of the type is inside an `UnsafeCell` or an atomic, and any bit pattern is a
/// valid value (`GamePtr::record` and `cells_of_bytes` rely on both). `cell_run` is safe to
/// get wrong (it hands out references): a wrong run is a wrong read, which the tests of every
/// store look for (`cells_tests`).
pub unsafe trait Cells {
    /// The run of like primitive cells holding byte `offset` (below the type's size), as
    /// references, and the byte it starts at: each field's own, an array of one primitive cell
    /// whole (`slice_run`).
    fn cell_run(&self, offset: usize) -> (usize, CellRun<'_>);

    /// The path of the field holding byte `offset` (`.a.b[2]`) added to `path`, and the type of
    /// the primitive cell it ends in: the debug pages' rows are named by it.
    #[cfg(any(test, feature = "write_watch"))]
    #[cfg_attr(not(debug_ui), allow(dead_code))] // only the debug pages' tests and the audit use it
    fn field_path(&self, _offset: usize, _path: &mut String) -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }

    /// A slice of this type as one run, when it is a primitive cell (an array's run).
    fn slice_run(_slice: &[Self]) -> Option<CellRun<'_>>
    where
        Self: Sized,
    {
        None
    }

    /// Every cell of this record set from the same cell of `other`, upwards (a float by its
    /// bits): a copy between two records of one type with no run walked. Records derive it
    /// field by field; this default walks the runs (`copy_cells`).
    fn copy_from(&self, other: &Self)
    where
        Self: Sized,
    {
        copy_cells((other, 0), (self, 0), size_of::<Self>());
    }

    /// The bytes of `now` that differ from the same bytes of `start` set into this record, the
    /// others left as they are: what one copy changed since it was `start`, put into another.
    /// Records derive it field by field; this default compares the bytes run by run.
    fn merge_changes(&self, now: &Self, start: &Self)
    where
        Self: Sized,
    {
        let size = size_of::<Self>();
        let (mut a, mut b) = (vec![0; size], vec![0; size]);
        // The whole record: inside it.
        let _ = load_cells(now, 0, &mut a);
        let _ = load_cells(start, 0, &mut b);
        let mut k = 0;
        while k < size {
            if a[k] == b[k] {
                k += 1;
                continue;
            }
            let first = k;
            while k < size && a[k] != b[k] {
                k += 1;
            }
            let _ = store_cells(self, first, &a[first..k]);
        }
    }
}

/// `now` when it differs from `start` (a cell's value changed, taken whole: two copies that
/// both rewrote a float must not leave one made of both's bytes), else none.
macro_rules! merged {
    ($t:ty, $now:expr, $start:expr) => {{
        let (now, start): ($t, $t) = ($now, $start);
        (now != start).then_some(now)
    }};
}

/// `len` bytes copied from byte `from.1` of one record of cells to byte `to.1` of another
/// (two different records, forwards as a `rep movsd`): the typed form of
/// `GamePtr::copy_to`, for two places known by their records, so no store is searched and
/// no address checked. Runs of like cells at aligned offsets are copied cell by cell (a
/// float by its bits); any other pair of runs through bytes.
///
/// # Panics
/// When a range lies outside its record.
pub fn copy_cells<F: Cells, T: Cells>(from: (&F, usize), to: (&T, usize), len: usize) {
    assert!(from.1 + len <= size_of::<F>() && to.1 + len <= size_of::<T>());
    let (mut a, mut b, mut left) = (from.1, to.1, len);
    while left > 0 {
        let (a_start, a_run) = from.0.cell_run(a);
        let (b_start, b_run) = to.0.cell_run(b);
        let (a_off, b_off) = (a - a_start, b - b_start);
        let n = left.min(a_run.len() - a_off).min(b_run.len() - b_off);
        copy_run_bytes(a_run, a_off, b_run, b_off, n);
        (a, b, left) = (a + n, b + n, left - n);
    }
}

/// `n` bytes from byte `a_off` of one run to byte `b_off` of another.
fn copy_run_bytes(from: CellRun, a_off: usize, to: CellRun, b_off: usize, n: usize) {
    let w = from.width();
    if w == to.width() && w > 1 && [a_off, b_off, n].iter().all(|x| x.is_multiple_of(w)) {
        for k in 0..n / w {
            let bytes = cell_bytes(from.cell(a_off / w + k));
            set_cell_bytes(to.cell(b_off / w + k), 0, &bytes[..w]);
        }
        return;
    }
    let mut buffer = [0u8; 256];
    let mut done = 0;
    while done < n {
        let part = &mut buffer[..(n - done).min(256)];
        from.load(a_off + done, part);
        to.store(b_off + done, part);
        done += part.len();
    }
}

/// One primitive cell, as its atomic.
#[derive(Clone, Copy)]
enum CellRef<'a> {
    U8(&'a AtomicU8),
    U16(&'a AtomicU16),
    U32(&'a AtomicU32),
    /// A non-atomic dword (a `PlainDword` and the other plain cells): only the game's thread
    /// reaches it.
    Plain32(&'a std::cell::Cell<u32>),
    /// A non-atomic byte (`PlainByte`, `PlainBytes`, `PlainLoose`).
    Plain8(&'a std::cell::Cell<u8>),
    /// A non-atomic word (`PlainWord`).
    Plain16(&'a std::cell::Cell<u16>),
}

impl CellRef<'_> {
    /// The cell noted for the write watch (`write_watch`).
    #[inline(always)]
    fn watched(self, write: bool) {
        let (at, len) = match self {
            CellRef::U8(a) => (a.as_ptr().cast_const(), 1),
            CellRef::U16(a) => (a.as_ptr().cast::<u8>().cast_const(), 2),
            CellRef::U32(a) => (a.as_ptr().cast::<u8>().cast_const(), 4),
            CellRef::Plain32(_) | CellRef::Plain8(_) | CellRef::Plain16(_) => return,
        };
        if write {
            crate::write_watch::wrote(at, len);
        } else {
            crate::write_watch::read(at, len);
        }
    }
}

/// A run of like primitive cells of a record, as references (`Cells::cell_run`): an array of
/// one cell type (or a single cell), a 4-aligned `Shared` record's dwords, or a run of byte
/// atomics (a `Bytes`). Copied in and out at its cells' width, a part of a cell merged by
/// compare-and-swap.
#[derive(Clone, Copy)]
pub enum CellRun<'a> {
    Bytes(&'a [AtomicU8]),
    Byte(&'a [crate::session::Byte]),
    Word(&'a [crate::session::Word]),
    Dword(&'a [crate::session::Dword]),
    Float(&'a [crate::session::Float]),
    Flag(&'a [crate::session::Flag]),
    Words(&'a [AtomicU32]),
    PlainDword(&'a [crate::session::PlainDword]),
    PlainFloat(&'a [crate::session::PlainFloat]),
    PlainFlag(&'a [crate::session::PlainFlag]),
    PlainBytes(&'a [std::cell::Cell<u8>]),
    PlainByte(&'a [crate::session::PlainByte]),
    PlainWord(&'a [crate::session::PlainWord]),
}

impl<'a> CellRun<'a> {
    /// Its cells' width in bytes.
    fn width(self) -> usize {
        match self {
            CellRun::Bytes(_)
            | CellRun::Byte(_)
            | CellRun::PlainBytes(_)
            | CellRun::PlainByte(_) => 1,
            CellRun::Word(_) | CellRun::PlainWord(_) => 2,
            CellRun::Dword(_)
            | CellRun::Float(_)
            | CellRun::Flag(_)
            | CellRun::Words(_)
            | CellRun::PlainDword(_)
            | CellRun::PlainFloat(_)
            | CellRun::PlainFlag(_) => 4,
        }
    }

    /// Its length in bytes.
    pub fn len(self) -> usize {
        let cells = match self {
            CellRun::Bytes(r) => r.len(),
            CellRun::Byte(r) => r.len(),
            CellRun::Word(r) => r.len(),
            CellRun::Dword(r) => r.len(),
            CellRun::Float(r) => r.len(),
            CellRun::Flag(r) => r.len(),
            CellRun::Words(r) => r.len(),
            CellRun::PlainDword(r) => r.len(),
            CellRun::PlainFloat(r) => r.len(),
            CellRun::PlainFlag(r) => r.len(),
            CellRun::PlainBytes(r) => r.len(),
            CellRun::PlainByte(r) => r.len(),
            CellRun::PlainWord(r) => r.len(),
        };
        cells * self.width()
    }

    /// Its first byte's address, for the tests' check of the tiling.
    #[cfg(test)]
    pub fn first(self) -> usize {
        match self {
            CellRun::Bytes(r) => r.as_ptr() as usize,
            CellRun::Byte(r) => r.as_ptr() as usize,
            CellRun::Word(r) => r.as_ptr() as usize,
            CellRun::Dword(r) => r.as_ptr() as usize,
            CellRun::Float(r) => r.as_ptr() as usize,
            CellRun::Flag(r) => r.as_ptr() as usize,
            CellRun::Words(r) => r.as_ptr() as usize,
            CellRun::PlainDword(r) => r.as_ptr() as usize,
            CellRun::PlainFloat(r) => r.as_ptr() as usize,
            CellRun::PlainFlag(r) => r.as_ptr() as usize,
            CellRun::PlainBytes(r) => r.as_ptr() as usize,
            CellRun::PlainByte(r) => r.as_ptr() as usize,
            CellRun::PlainWord(r) => r.as_ptr() as usize,
        }
    }

    /// The run from its byte `n` on, when that is a cell's first byte.
    pub fn skip(self, n: usize) -> Option<CellRun<'a>> {
        if !n.is_multiple_of(self.width()) || n > self.len() {
            return None;
        }
        let i = n / self.width();
        Some(match self {
            CellRun::Bytes(r) => CellRun::Bytes(&r[i..]),
            CellRun::Byte(r) => CellRun::Byte(&r[i..]),
            CellRun::Word(r) => CellRun::Word(&r[i..]),
            CellRun::Dword(r) => CellRun::Dword(&r[i..]),
            CellRun::Float(r) => CellRun::Float(&r[i..]),
            CellRun::Flag(r) => CellRun::Flag(&r[i..]),
            CellRun::Words(r) => CellRun::Words(&r[i..]),
            CellRun::PlainDword(r) => CellRun::PlainDword(&r[i..]),
            CellRun::PlainFloat(r) => CellRun::PlainFloat(&r[i..]),
            CellRun::PlainFlag(r) => CellRun::PlainFlag(&r[i..]),
            CellRun::PlainBytes(r) => CellRun::PlainBytes(&r[i..]),
            CellRun::PlainByte(r) => CellRun::PlainByte(&r[i..]),
            CellRun::PlainWord(r) => CellRun::PlainWord(&r[i..]),
        })
    }

    /// Cell `i`.
    fn cell(self, i: usize) -> CellRef<'a> {
        match self {
            CellRun::Bytes(r) => CellRef::U8(&r[i]),
            CellRun::Byte(r) => CellRef::U8(r[i].atomic()),
            CellRun::Word(r) => CellRef::U16(r[i].atomic()),
            CellRun::Dword(r) => CellRef::U32(r[i].atomic()),
            CellRun::Float(r) => CellRef::U32(r[i].atomic()),
            CellRun::Flag(r) => CellRef::U32(r[i].atomic()),
            CellRun::Words(r) => CellRef::U32(&r[i]),
            CellRun::PlainDword(r) => CellRef::Plain32(r[i].cell()),
            CellRun::PlainFloat(r) => CellRef::Plain32(r[i].cell()),
            CellRun::PlainFlag(r) => CellRef::Plain32(r[i].cell()),
            CellRun::PlainBytes(r) => CellRef::Plain8(&r[i]),
            CellRun::PlainByte(r) => CellRef::Plain8(r[i].cell()),
            CellRun::PlainWord(r) => CellRef::Plain16(r[i].cell()),
        }
    }

    /// `to.len()` bytes from its byte `offset` (inside it), each cell read whole.
    pub fn load(self, offset: usize, to: &mut [u8]) {
        if let CellRun::Bytes(r) = self {
            return load_bytes(&r[offset..offset + to.len()], to);
        }
        if let CellRun::PlainBytes(r) = self {
            for (b, c) in to.iter_mut().zip(&r[offset..]) {
                *b = c.get();
            }
            return;
        }
        let (w, end) = (self.width(), offset + to.len());
        let mut at = offset;
        while at < end {
            let start = at - at % w;
            let upto = end.min(start + w);
            let bytes = cell_bytes(self.cell(start / w));
            to[at - offset..upto - offset].copy_from_slice(&bytes[at - start..upto - start]);
            at = upto;
        }
    }

    /// `from` written at its byte `offset` (inside it), each cell at its own width.
    pub fn store(self, offset: usize, from: &[u8]) {
        if let CellRun::Bytes(r) = self {
            return store_bytes(&r[offset..offset + from.len()], from);
        }
        if let CellRun::PlainBytes(r) = self {
            for (c, &b) in r[offset..].iter().zip(from) {
                c.set(b);
            }
            return;
        }
        let (w, end) = (self.width(), offset + from.len());
        let mut at = offset;
        while at < end {
            let start = at - at % w;
            let upto = end.min(start + w);
            set_cell_bytes(
                self.cell(start / w),
                at - start,
                &from[at - offset..upto - offset],
            );
            at = upto;
        }
    }
}

// SAFETY: one `AtomicU32`.
unsafe impl Cells for crate::session::Dword {
    fn copy_from(&self, other: &Self) {
        self.atomic().store(other.atomic().load(Relaxed), Relaxed);
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        let a = self.atomic();
        if let Some(v) = merged!(
            u32,
            now.atomic().load(Relaxed),
            start.atomic().load(Relaxed)
        ) {
            a.store(v, Relaxed);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Dword(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Dword(slice))
    }
}
// SAFETY: one `AtomicU32`.
unsafe impl Cells for crate::session::Float {
    fn copy_from(&self, other: &Self) {
        self.atomic().store(other.atomic().load(Relaxed), Relaxed);
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        let a = self.atomic();
        if let Some(v) = merged!(
            u32,
            now.atomic().load(Relaxed),
            start.atomic().load(Relaxed)
        ) {
            a.store(v, Relaxed);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Float(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Float(slice))
    }
}
// SAFETY: one `AtomicU32`.
unsafe impl Cells for crate::session::Flag {
    fn copy_from(&self, other: &Self) {
        self.atomic().store(other.atomic().load(Relaxed), Relaxed);
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        let a = self.atomic();
        if let Some(v) = merged!(
            u32,
            now.atomic().load(Relaxed),
            start.atomic().load(Relaxed)
        ) {
            a.store(v, Relaxed);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Flag(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Flag(slice))
    }
}
// SAFETY: one `Cell<u32>`.
unsafe impl Cells for crate::session::PlainDword {
    fn copy_from(&self, other: &Self) {
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u32, now.cell().get(), start.cell().get()) {
            self.cell().set(v);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainDword(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainDword(slice))
    }
}
// SAFETY: one `Cell<u32>`.
unsafe impl Cells for crate::session::PlainFloat {
    fn copy_from(&self, other: &Self) {
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u32, now.cell().get(), start.cell().get()) {
            self.cell().set(v);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainFloat(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainFloat(slice))
    }
}
// SAFETY: one `Cell<u32>`.
unsafe impl Cells for crate::session::PlainFlag {
    fn copy_from(&self, other: &Self) {
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u32, now.cell().get(), start.cell().get()) {
            self.cell().set(v);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainFlag(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainFlag(slice))
    }
}

// SAFETY: `first` is the record's byte at the window's offset and `size` what is left of it;
// the data is reached only after the game-thread check.
unsafe impl<S: Cells> Store for crate::session::OnGameThread<'_, S> {
    fn size(&self) -> usize {
        size_of::<S>() - self.offset
    }

    fn cell_run_at(&self, offset: usize) -> Option<(usize, CellRun<'_>)> {
        crate::session::assert_game_thread();
        // A run that begins before the window is taken from the window's first byte, when a
        // cell begins there.
        let (start, run) = Cells::cell_run(self.whole, self.offset + offset);
        match start.checked_sub(self.offset) {
            Some(start) => Some((start, run)),
            None => Some((0, run.skip(self.offset - start)?)),
        }
    }

    #[cfg(any(test, feature = "write_watch"))]
    fn field_path_at(&self, offset: usize, path: &mut String) -> Option<&'static str> {
        Some(Cells::field_path(self.whole, self.offset + offset, path))
    }

    #[cfg(any(test, feature = "write_watch"))]
    fn type_name(&self) -> &'static str {
        std::any::type_name::<S>()
    }

    fn first(&self) -> *const u8 {
        std::ptr::from_ref(self.whole)
            .cast::<u8>()
            .wrapping_add(self.offset)
    }

    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), BadInput> {
        crate::session::assert_game_thread();
        load_cells(self.whole, self.offset + offset, to)
    }

    fn store(&self, offset: usize, from: &[u8]) -> Result<(), BadInput> {
        crate::session::assert_game_thread();
        store_cells(self.whole, self.offset + offset, from)
    }
}
// SAFETY: one `AtomicU16`.
unsafe impl Cells for crate::session::Word {
    fn copy_from(&self, other: &Self) {
        self.atomic().store(other.atomic().load(Relaxed), Relaxed);
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        let a = self.atomic();
        if let Some(v) = merged!(
            u16,
            now.atomic().load(Relaxed),
            start.atomic().load(Relaxed)
        ) {
            a.store(v, Relaxed);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Word(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Word(slice))
    }
}
// SAFETY: one `AtomicU8`.
unsafe impl Cells for crate::session::Byte {
    fn copy_from(&self, other: &Self) {
        self.atomic().store(other.atomic().load(Relaxed), Relaxed);
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        let a = self.atomic();
        if let Some(v) = merged!(u8, now.atomic().load(Relaxed), start.atomic().load(Relaxed)) {
            a.store(v, Relaxed);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Byte(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Byte(slice))
    }
}
// SAFETY: an `UnsafeCell` over plain bytes, all of it one run.
unsafe impl<const N: usize> Cells for crate::session::Bytes<N> {
    fn copy_from(&self, other: &Self) {
        for (to, from) in self.atomics().iter().zip(other.atomics()) {
            to.store(from.load(Relaxed), Relaxed);
        }
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        for ((to, now), start) in self
            .atomics()
            .iter()
            .zip(now.atomics())
            .zip(start.atomics())
        {
            let v = now.load(Relaxed);
            if v != start.load(Relaxed) {
                to.store(v, Relaxed);
            }
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Bytes(self.atomics()))
    }
}
// SAFETY: an `UnsafeCell` over a plain record (any bytes make one, `Plain`), at the widths
// its `get` / `set` take (`Shared::WORDS`): a 4-aligned record's whole dwords one run of
// `AtomicU32`s, the bytes past them `AtomicU8`s; a packed record one run of shared bytes.
unsafe impl<T: Plain> Cells for crate::session::Shared<T> {
    fn cell_run(&self, offset: usize) -> (usize, CellRun<'_>) {
        crate::session::Shared::cell_run(self, offset)
    }
}
// SAFETY: an `UnsafeCell` over four plain bytes, all of it one run.
unsafe impl Cells for crate::session::Loose {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Bytes(self.atomics()))
    }
}
// SAFETY: one `Cell<u16>`.
unsafe impl Cells for crate::session::PlainWord {
    fn copy_from(&self, other: &Self) {
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u16, now.cell().get(), start.cell().get()) {
            self.cell().set(v);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainWord(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainWord(slice))
    }
}
// SAFETY: one `Cell<u8>`.
unsafe impl Cells for crate::session::PlainByte {
    fn copy_from(&self, other: &Self) {
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u8, now.cell().get(), start.cell().get()) {
            self.cell().set(v);
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainByte(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainByte(slice))
    }
}
// SAFETY: `N` cells of plain bytes, all of it one run.
unsafe impl<const N: usize> Cells for crate::session::PlainBytes<N> {
    fn copy_from(&self, other: &Self) {
        for (to, from) in self.cells().iter().zip(other.cells()) {
            to.set(from.get());
        }
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        for ((to, now), start) in self.cells().iter().zip(now.cells()).zip(start.cells()) {
            let v = now.get();
            if v != start.get() {
                to.set(v);
            }
        }
    }

    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainBytes(self.cells()))
    }
}
// SAFETY: four cells of plain bytes, all of it one run.
unsafe impl Cells for crate::session::PlainLoose {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainBytes(self.cells()))
    }
}
// SAFETY: `N` of a type of cells, one after another.
unsafe impl<T: Cells, const N: usize> Cells for [T; N] {
    fn copy_from(&self, other: &Self) {
        for (to, from) in self.iter().zip(other) {
            to.copy_from(from);
        }
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        for ((to, now), start) in self.iter().zip(now).zip(start) {
            to.merge_changes(now, start);
        }
    }

    #[cfg(any(test, feature = "write_watch"))]
    fn field_path(&self, offset: usize, path: &mut String) -> &'static str {
        let size = size_of::<T>();
        path.push_str(&format!("[{}]", offset / size));
        self[offset / size].field_path(offset % size, path)
    }

    fn cell_run(&self, offset: usize) -> (usize, CellRun<'_>) {
        if let Some(run) = T::slice_run(self.as_slice()) {
            return (0, run);
        }
        let size = size_of::<T>();
        let i = offset / size;
        let (start, run) = self[i].cell_run(offset % size);
        (i * size + start, run)
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
pub(crate) fn zeroed<T: Plain>() -> T {
    // SAFETY: any bytes make a `T` (`Plain`), zeros among them.
    unsafe { std::mem::zeroed() }
}

/// The bytes of a `T`, to be read into.
pub(crate) fn bytes_mut<T: Plain>(value: &mut T) -> &mut [u8] {
    // SAFETY: a `T` has no padding and any bytes make one (`Plain`); the slice borrows it.
    unsafe { std::slice::from_raw_parts_mut(std::ptr::from_mut(value).cast(), size_of::<T>()) }
}

/// The bytes of a `T`.
pub(crate) fn bytes_of<T: Plain>(value: &T) -> &[u8] {
    // SAFETY: a `T` has no padding (`Plain`); the slice borrows it.
    unsafe { std::slice::from_raw_parts(std::ptr::from_ref(value).cast(), size_of::<T>()) }
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
/// `first()` is the store's first byte and `size()` bytes behind it are the store's own, laid
/// out as it says (`GamePtr::record` and `slice` reach them from `first()`).
pub unsafe trait Store {
    /// Its length in bytes.
    fn size(&self) -> usize;
    /// Its first byte: a checked place's bytes are found from it (`place_of`), inside `size()`.
    fn first(&self) -> *const u8;
    /// The run of like cells holding byte `offset`, as references, and the byte it starts at
    /// (`None`: the reads there take `load` and `store`).
    fn cell_run_at(&self, _offset: usize) -> Option<(usize, CellRun<'_>)> {
        None
    }
    /// `Cells::field_path` of the record behind the store, and the record's type.
    #[cfg(any(test, feature = "write_watch"))]
    #[cfg_attr(not(debug_ui), allow(dead_code))] // only the debug pages' tests and the audit use it
    fn field_path_at(&self, _offset: usize, _path: &mut String) -> Option<&'static str> {
        None
    }
    #[cfg(any(test, feature = "write_watch"))]
    #[cfg_attr(not(feature = "write_watch"), allow(dead_code))] // only the audit uses it
    fn type_name(&self) -> &'static str {
        ""
    }
    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), BadInput>;
    fn store(&self, offset: usize, from: &[u8]) -> Result<(), BadInput>;
}

// SAFETY: `first` is the record's first byte and `size` its size.
unsafe impl<T: Cells> Store for T {
    fn size(&self) -> usize {
        size_of::<T>()
    }

    fn cell_run_at(&self, offset: usize) -> Option<(usize, CellRun<'_>)> {
        Some(Cells::cell_run(self, offset))
    }

    #[cfg(any(test, feature = "write_watch"))]
    fn field_path_at(&self, offset: usize, path: &mut String) -> Option<&'static str> {
        Some(Cells::field_path(self, offset, path))
    }

    #[cfg(any(test, feature = "write_watch"))]
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
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
fn cell_bytes(cell: CellRef) -> [u8; 4] {
    use std::sync::atomic::Ordering::Relaxed;
    cell.watched(false);
    let mut bytes = [0; 4];
    match cell {
        CellRef::U8(a) => bytes[0] = a.load(Relaxed),
        CellRef::U16(a) => bytes[..2].copy_from_slice(&a.load(Relaxed).to_ne_bytes()),
        CellRef::U32(a) => bytes = a.load(Relaxed).to_ne_bytes(),
        CellRef::Plain32(c) => bytes = c.get().to_ne_bytes(),
        CellRef::Plain8(c) => bytes[0] = c.get(),
        CellRef::Plain16(c) => bytes[..2].copy_from_slice(&c.get().to_ne_bytes()),
    }
    bytes
}

/// A cell's bytes `from ..` set to `new`, the others kept: one store when all of it is new,
/// else one compare-and-swap, so a byte written into a dword never undoes a write to the
/// `new` written into a cell from its byte `from`: a whole cell stored, a part of one merged
/// by one compare-and-swap (the other bytes kept as another thread left them).
fn set_cell_bytes(cell: CellRef, from: usize, new: &[u8]) {
    use std::sync::atomic::Ordering::Relaxed;
    cell.watched(true);
    let merge = |old: [u8; 4]| {
        let mut bytes = old;
        bytes[from..from + new.len()].copy_from_slice(new);
        bytes
    };
    match cell {
        CellRef::U8(a) => a.store(new[0], Relaxed),
        CellRef::U16(a) => {
            let _ = a.try_update(Relaxed, Relaxed, |old| {
                let [a, b, ..] = old.to_ne_bytes();
                let [a, b, ..] = merge([a, b, 0, 0]);
                Some(u16::from_ne_bytes([a, b]))
            });
        }
        CellRef::Plain32(c) => c.set(u32::from_ne_bytes(merge(c.get().to_ne_bytes()))),
        CellRef::Plain8(c) => c.set(new[0]),
        CellRef::Plain16(c) => {
            let [a, b, ..] = c.get().to_ne_bytes();
            let [a, b, ..] = merge([a, b, 0, 0]);
            c.set(u16::from_ne_bytes([a, b]));
        }
        CellRef::U32(a) => {
            if new.len() == 4 {
                a.store(u32::from_ne_bytes(merge([0; 4])), Relaxed);
            } else {
                let _ = a.try_update(Relaxed, Relaxed, |old| {
                    Some(u32::from_ne_bytes(merge(old.to_ne_bytes())))
                });
            }
        }
    }
}

/// `to.len()` bytes of `record` from byte `offset` on, each cell read whole at its own
/// width (a run of `Bytes` as `shared_memory` reads it).
pub fn load_cells<T: Cells>(record: &T, offset: usize, to: &mut [u8]) -> Result<(), BadInput> {
    let end = record_end::<T>(offset, to.len())?;
    let mut at = offset;
    while at < end {
        let (start, run) = record.cell_run(at);
        let upto = end.min(start + run.len());
        run.load(at - start, &mut to[at - offset..upto - offset]);
        at = upto;
    }
    Ok(())
}

/// `from` written into `record` from byte `offset` on, each cell at its own width (a part
/// of one merged by compare-and-swap).
pub fn store_cells<T: Cells>(record: &T, offset: usize, from: &[u8]) -> Result<(), BadInput> {
    let end = record_end::<T>(offset, from.len())?;
    let mut at = offset;
    while at < end {
        let (start, run) = record.cell_run(at);
        let upto = end.min(start + run.len());
        run.store(at - start, &from[at - offset..upto - offset]);
        at = upto;
    }
    Ok(())
}

/// The end of `len` bytes from `offset` in a `T`, or refused past its end.
fn record_end<T>(offset: usize, len: usize) -> Result<usize, BadInput> {
    offset
        .checked_add(len)
        .filter(|&end| end <= size_of::<T>())
        .ok_or(BadInput("past the end of a record"))
}

/// A run of byte atomics copied out.
fn load_bytes(run: &[AtomicU8], to: &mut [u8]) {
    if let Some(a) = run.first() {
        crate::write_watch::read(a.as_ptr().cast_const(), run.len().min(to.len()));
    }
    for (b, a) in to.iter_mut().zip(run) {
        *b = a.load(std::sync::atomic::Ordering::Relaxed);
    }
}

/// A run of byte atomics written.
fn store_bytes(run: &[AtomicU8], from: &[u8]) {
    if let Some(a) = run.first() {
        crate::write_watch::wrote(a.as_ptr().cast_const(), run.len().min(from.len()));
    }
    for (a, &b) in run.iter().zip(from) {
        a.store(b, std::sync::atomic::Ordering::Relaxed);
    }
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

/// A store's bytes as a reader (`rc2k_core::strings::bytes::Bytes`): the field's own bytes, none
/// past its end.
pub struct StoreBytes<'a>(pub &'a dyn Store);

impl rc2k_core::strings::bytes::Bytes for StoreBytes<'_> {
    fn byte(&self, at: usize) -> Option<u8> {
        let mut byte = [0];
        self.copy_run(at, &mut byte)?;
        Some(byte[0])
    }
    fn copy_run(&self, at: usize, out: &mut [u8]) -> Option<()> {
        let end = at.checked_add(out.len())?;
        (end <= self.0.size() && self.0.load(at, out).is_ok()).then_some(())
    }
}

/// The `T` at byte `offset` of `store` (any alignment; a run past the store is refused).
pub fn load_value<T: Plain>(store: &dyn Store, offset: usize) -> Result<T, BadInput> {
    let mut value = zeroed::<T>();
    store.load(offset, bytes_mut(&mut value))?;
    Ok(value)
}

/// `value` written at byte `offset` of `store` (a run past the store is refused).
pub fn store_value<T: Plain>(store: &dyn Store, offset: usize, value: T) -> Result<(), BadInput> {
    store.store(offset, bytes_of(&value))
}

/// `len` bytes copied from `from` at `from_at` to `to` at `to_at`: each side inside its own store
/// (a run past either is refused, naming `what`).
pub fn copy_between(
    from: &dyn Store,
    from_at: usize,
    to: &dyn Store,
    to_at: usize,
    len: usize,
    what: &str,
) -> Result<(), BadInput> {
    let mut bytes = vec![0; len];
    from.load(from_at, &mut bytes).inspect_err(|_| {
        log(&format!(
            "pointers: {what}: {len} bytes read at {from_at:#x}"
        ));
    })?;
    to.store(to_at, &bytes).inspect_err(|_| {
        log(&format!(
            "pointers: {what}: {len} bytes written at {to_at:#x}"
        ));
    })
}

#[cfg(test)]
mod tests;
