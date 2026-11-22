//! The cells the game's data lives in: `Cells` (a record of atomics and cells, copied, merged
//! and visited field by field in any layout), `ByteView` (a record of fixed layout whose runs
//! are found by byte offset), its runs and references, `Plain` values, `Store` (what a byte
//! offset resolves to), and the copies between them.
//!
//! `ByteView::cell_run` and `CellRun`: the byte-wise copies, `load_cells` and
//! `checkpoints_store::run_on` walk runs. `ByteView::field_path` is for the tests.

#![allow(unsafe_code)]

use crate::errors::BadInput;
use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, Ordering::Relaxed};

/// A type made of atomics and cells only (`session::{Dword, Float, Word, Byte, Bytes<N>}`, their
/// arrays, records of them in any layout), which may be handed out shared.
///
/// The record is copied, merged and visited field by field; it promises no byte layout. The
/// byte view (`ByteView`) is the trait of the records whose layout is fixed.
///
/// # Safety
/// Every byte of the type that holds a value is inside an `UnsafeCell` or an atomic, and the
/// all-zero value is valid (`zeroed_cells` relies on both).
pub unsafe trait Cells {
    /// Every cell of this record set from the same cell of `other`, upwards (a float by its
    /// bits): a copy between two records of one type with no run walked. Records derive it
    /// field by field.
    fn copy_from(&self, other: &Self)
    where
        Self: Sized;

    /// The cells of `now` that differ from the same cells of `start` set into this record, the
    /// others left as they are: what one copy changed since it was `start`, put into another.
    /// Records derive it field by field.
    fn merge_changes(&self, now: &Self, start: &Self)
    where
        Self: Sized;

    /// Every primitive run of this record handed to `visitor`, with the field names and array
    /// indices on the way, with no byte layout needed by the caller. Records derive it field
    /// by field; the primitive cells report themselves as one run.
    fn visit(&self, visitor: &mut dyn CellVisitor)
    where
        Self: Sized;

    /// A whole slice of this type reported as one run, when it is a primitive cell (an array
    /// of them is one run, element 0): `index(0)`, the leaf, `leave`. Says whether it did.
    fn visit_slice(_slice: &[Self], _visitor: &mut dyn CellVisitor) -> bool
    where
        Self: Sized,
    {
        false
    }
}

/// A record of fixed layout, whose bytes can be reached by offset: its runs of like primitive
/// cells (`cell_run`), the path of a byte's field. Stores are made of these.
///
/// # Safety
/// The type is `#[repr(C)]` (or transparent) over `ByteView` fields with no padding, so any bit
/// pattern of `size_of::<Self>()` bytes is a valid value. Every
/// byte is inside an `UnsafeCell` or an atomic (`Cells`). `cell_run` is safe to get wrong (it
/// hands out references): a wrong run is a wrong read, which the tests of every store look for
/// (`cells_tests`).
pub unsafe trait ByteView: Cells {
    /// The run of like primitive cells holding byte `offset` (below the type's size), as
    /// references, and the byte it starts at: each field's own, an array of one primitive cell
    /// whole (`slice_run`).
    fn cell_run(&self, offset: usize) -> (usize, CellRun<'_>);

    /// The path of the field holding byte `offset` (`.a.b[2]`) added to `path`, and the type of
    /// the primitive cell it ends in: the debug pages' rows are named by it.
    #[cfg(test)]
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
}

/// Every run of a record whose cells are found by byte (`Shared`, `Loose`) handed to `visitor`:
/// its runs from byte 0 on, each as one leaf.
fn visit_by_runs<T: ByteView>(record: &T, visitor: &mut dyn CellVisitor) {
    let size = size_of::<T>();
    let mut at = 0;
    while at < size {
        let (start, run) = record.cell_run(at);
        let end = (start + run.len()).min(size);
        if at == start && end == start + run.len() {
            visitor.run(run);
        } else {
            let mut bytes = vec![0; end - at];
            run.load(at - start, &mut bytes);
            visitor.leaf(&bytes);
        }
        at = end;
    }
}

/// One run reported to the visitor.
fn visit_run(run: CellRun, visitor: &mut dyn CellVisitor) {
    if run.len() == 0 {
        return;
    }
    visitor.run(run);
}

/// A whole slice of primitive cells reported as element 0 of an array.
fn visit_cells_slice(run: CellRun, visitor: &mut dyn CellVisitor) -> bool {
    visitor.index(0);
    visit_run(run, visitor);
    visitor.leave();
    true
}

/// Every byte of `record` set from the same byte of `other`, by runs: `Cells::copy_from` of a
/// record that has no fields to copy (`Shared`, `Loose`).
fn copy_by_runs<T: ByteView>(to: &T, from: &T) {
    copy_cells((from, 0), (to, 0), size_of::<T>());
}

/// The bytes of `now` that differ from the same bytes of `start` set into `to`, the others left
/// as they are, compared run by run: `Cells::merge_changes` of a record with no fields.
fn merge_by_runs<T: ByteView>(to: &T, now: &T, start: &T) {
    let size = size_of::<T>();
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
        let _ = store_cells(to, first, &a[first..k]);
    }
}

/// What `Cells::visit` reports: the fields it goes through (`enter` .. `leave`, an array's
/// element as `index` .. `leave`; a field named `inline` is not entered) and each run of like
/// primitive cells with its bytes (a cell's value in little-endian bytes, a float by its bits).
pub trait CellVisitor {
    /// A field called `name` begins.
    fn enter(&mut self, name: &'static str);
    /// Element `i` of an array begins (a run of primitive cells is element 0 as a whole).
    fn index(&mut self, i: usize);
    /// The field or element entered last ends.
    fn leave(&mut self);
    /// A run of like primitive cells, its bytes.
    fn leaf(&mut self, bytes: &[u8]);
    /// A run of like primitive cells, as the cells themselves: a visitor that reads or writes
    /// them, or wants their addresses, overrides this. The default reports the run's bytes
    /// through `leaf`.
    fn run(&mut self, run: CellRun<'_>) {
        let mut bytes = vec![0; run.len()];
        run.load(0, &mut bytes);
        self.leaf(&bytes);
    }
}

/// A record, array or cell copied and merged into another of the same shape in other cells
/// (`session::CellKind`: the game thread's plain cells and shared atomics), field by field and
/// cell by cell (a float by its bits), with no byte layout. Records over a kind derive it
/// (`#[derive(Cells)]`); primitive 32-bit cells and arrays of them implement it here.
pub trait SameShape<To: ?Sized> {
    /// Every cell of `to` set from the same cell of `self`, in field order.
    fn copy_cells_to(&self, to: &To);

    /// The cells of `self` that differ from the same cells of `start` set into `to`, the
    /// others left as they are; each cell is compared and taken whole.
    #[allow(
        dead_code,
        reason = "the frame views' hand-offs merge between same-type records"
    )]
    fn merge_cells_to(&self, start: &Self, to: &To);
}

impl<A: crate::memory::session::Cell32, B: crate::memory::session::Cell32> SameShape<B> for A {
    fn copy_cells_to(&self, to: &B) {
        to.set_bits(self.bits());
    }

    fn merge_cells_to(&self, start: &Self, to: &B) {
        let now = self.bits();
        if now != start.bits() {
            to.set_bits(now);
        }
    }
}

impl<A: SameShape<B>, B, const N: usize> SameShape<[B; N]> for [A; N] {
    fn copy_cells_to(&self, to: &[B; N]) {
        for (from, to) in self.iter().zip(to) {
            from.copy_cells_to(to);
        }
    }

    fn merge_cells_to(&self, start: &Self, to: &[B; N]) {
        for ((now, start), to) in self.iter().zip(start).zip(to) {
            now.merge_cells_to(start, to);
        }
    }
}

/// Every primitive run's bytes of `record` one after another, whatever kinds of cells hold
/// them: two records of one shape in other cells give equal bytes when their values are equal.
pub fn flatten<T: Cells>(record: &T) -> Vec<u8> {
    struct Flat(Vec<u8>);
    impl CellVisitor for Flat {
        fn enter(&mut self, _: &'static str) {}
        fn index(&mut self, _: usize) {}
        fn leave(&mut self) {}
        fn leaf(&mut self, bytes: &[u8]) {
            self.0.extend_from_slice(bytes);
        }
    }
    let mut flat = Flat(Vec::new());
    record.visit(&mut flat);
    flat.0
}

/// `now` when it differs from `start` (a cell's value changed, taken whole: two copies that
/// both rewrote a float must not leave one made of both's bytes), else none.
macro_rules! merged {
    ($t:ty, $now:expr, $start:expr) => {{
        let (now, start): ($t, $t) = ($now, $start);
        (now != start).then_some(now)
    }};
}

/// `len` bytes copied from byte `from.1` of one record of cells to byte `to.1` of another (two
/// different records, forwards), for two places known by their records, so no store is searched.
/// Runs of like cells at aligned offsets are copied cell by cell (a float by its bits); any other
/// pair of runs through bytes.
///
/// # Panics
/// When a range lies outside its record.
pub fn copy_cells<F: ByteView, T: ByteView>(from: (&F, usize), to: (&T, usize), len: usize) {
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

/// The whole of one record copied into another of the same layout in other cells (a store's
/// plain record into an atomic copy of it, `session::CellKind`), cell by cell (a float by its
/// bits).
///
/// # Panics
/// When the two differ in size.
#[cfg(test)]
fn copy_whole<F: ByteView, T: ByteView>(from: &F, to: &T) {
    assert_eq!(size_of::<F>(), size_of::<T>(), "two records of one layout");
    copy_cells((from, 0), (to, 0), size_of::<F>());
}

/// The cells of `now` that differ from the same cells of `start` set into `to`, the others
/// left as they are: `Cells::merge_changes` between records of the same layout in other cells
/// (`now` and `start` an atomic copy of the plain `to`, `session::CellKind`). Each cell is
/// compared and taken whole, as the derived merge does.
///
/// # Panics
/// When the records differ in size, or their cells do not line up.
#[cfg(test)]
fn merge_cell_changes<T: ByteView, F: ByteView>(to: &T, now: &F, start: &F) {
    let size = size_of::<T>();
    assert_eq!(size, size_of::<F>(), "two records of one layout");
    let mut at = 0;
    while at < size {
        let (to_start, to_run) = to.cell_run(at);
        let (now_start, now_run) = now.cell_run(at);
        let (_, start_run) = start.cell_run(at);
        let w = to_run.width();
        assert!(
            w == now_run.width() && to_start == at && now_start == at,
            "the records' cells line up"
        );
        let n = to_run.len().min(now_run.len());
        for k in 0..n / w {
            let a = cell_bytes(now_run.cell(k));
            if a != cell_bytes(start_run.cell(k)) {
                set_cell_bytes(to_run.cell(k), 0, &a[..w]);
            }
        }
        at += n;
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
    /// A non-atomic byte (`PlainByte`, `PlainBytes`).
    Plain8(&'a std::cell::Cell<u8>),
    /// A non-atomic word (`PlainWord`).
    Plain16(&'a std::cell::Cell<u16>),
}

/// A run of like primitive cells of a record, as references (`Cells::cell_run`): an array of
/// one cell type (or a single cell), a 4-aligned `Shared` record's dwords, or a run of byte
/// atomics (a `Bytes`). Copied in and out at its cells' width, a part of a cell merged by
/// compare-and-swap.
#[derive(Clone, Copy)]
pub enum CellRun<'a> {
    Bytes(&'a [AtomicU8]),
    Byte(&'a [crate::memory::session::Byte]),
    Word(&'a [crate::memory::session::Word]),
    Dword(&'a [crate::memory::session::Dword]),
    Float(&'a [crate::memory::session::Float]),
    Int(&'a [crate::memory::session::Int]),
    Flag(&'a [crate::memory::session::Flag]),
    Words(&'a [AtomicU32]),
    PlainDword(&'a [crate::memory::session::PlainDword]),
    PlainFloat(&'a [crate::memory::session::PlainFloat]),
    PlainFlag(&'a [crate::memory::session::PlainFlag]),
    PlainInt(&'a [crate::memory::session::PlainInt]),
    PlainBytes(&'a [std::cell::Cell<u8>]),
    PlainByte(&'a [crate::memory::session::PlainByte]),
    PlainWord(&'a [crate::memory::session::PlainWord]),
    /// The words of a `PlainFloats`.
    PlainWords(&'a [std::cell::Cell<u32>]),
}

impl<'a> CellRun<'a> {
    /// Its cells' width in bytes.
    pub fn width(self) -> usize {
        match self {
            CellRun::Bytes(_)
            | CellRun::Byte(_)
            | CellRun::PlainBytes(_)
            | CellRun::PlainByte(_) => 1,
            CellRun::Word(_) | CellRun::PlainWord(_) => 2,
            CellRun::Dword(_)
            | CellRun::Float(_)
            | CellRun::Int(_)
            | CellRun::Flag(_)
            | CellRun::Words(_)
            | CellRun::PlainDword(_)
            | CellRun::PlainFloat(_)
            | CellRun::PlainFlag(_)
            | CellRun::PlainInt(_)
            | CellRun::PlainWords(_) => 4,
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
            CellRun::Int(r) => r.len(),
            CellRun::Flag(r) => r.len(),
            CellRun::Words(r) => r.len(),
            CellRun::PlainDword(r) => r.len(),
            CellRun::PlainFloat(r) => r.len(),
            CellRun::PlainFlag(r) => r.len(),
            CellRun::PlainInt(r) => r.len(),
            CellRun::PlainBytes(r) => r.len(),
            CellRun::PlainByte(r) => r.len(),
            CellRun::PlainWord(r) => r.len(),
            CellRun::PlainWords(r) => r.len(),
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
            CellRun::Int(r) => r.as_ptr() as usize,
            CellRun::Flag(r) => r.as_ptr() as usize,
            CellRun::Words(r) => r.as_ptr() as usize,
            CellRun::PlainDword(r) => r.as_ptr() as usize,
            CellRun::PlainFloat(r) => r.as_ptr() as usize,
            CellRun::PlainFlag(r) => r.as_ptr() as usize,
            CellRun::PlainInt(r) => r.as_ptr() as usize,
            CellRun::PlainBytes(r) => r.as_ptr() as usize,
            CellRun::PlainByte(r) => r.as_ptr() as usize,
            CellRun::PlainWord(r) => r.as_ptr() as usize,
            CellRun::PlainWords(r) => r.as_ptr() as usize,
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
            CellRun::Int(r) => CellRun::Int(&r[i..]),
            CellRun::Flag(r) => CellRun::Flag(&r[i..]),
            CellRun::Words(r) => CellRun::Words(&r[i..]),
            CellRun::PlainDword(r) => CellRun::PlainDword(&r[i..]),
            CellRun::PlainFloat(r) => CellRun::PlainFloat(&r[i..]),
            CellRun::PlainFlag(r) => CellRun::PlainFlag(&r[i..]),
            CellRun::PlainInt(r) => CellRun::PlainInt(&r[i..]),
            CellRun::PlainBytes(r) => CellRun::PlainBytes(&r[i..]),
            CellRun::PlainByte(r) => CellRun::PlainByte(&r[i..]),
            CellRun::PlainWord(r) => CellRun::PlainWord(&r[i..]),
            CellRun::PlainWords(r) => CellRun::PlainWords(&r[i..]),
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
            CellRun::Int(r) => CellRef::U32(r[i].atomic()),
            CellRun::Flag(r) => CellRef::U32(r[i].atomic()),
            CellRun::Words(r) => CellRef::U32(&r[i]),
            CellRun::PlainDword(r) => CellRef::Plain32(r[i].cell()),
            CellRun::PlainFloat(r) => CellRef::Plain32(r[i].cell()),
            CellRun::PlainFlag(r) => CellRef::Plain32(r[i].cell()),
            CellRun::PlainInt(r) => CellRef::Plain32(r[i].cell()),
            CellRun::PlainBytes(r) => CellRef::Plain8(&r[i]),
            CellRun::PlainByte(r) => CellRef::Plain8(r[i].cell()),
            CellRun::PlainWord(r) => CellRef::Plain16(r[i].cell()),
            CellRun::PlainWords(r) => CellRef::Plain32(&r[i]),
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
                crate::memory::session::plain_written(c);
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

// SAFETY: `repr(transparent)` over one `AtomicU32`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::Dword {
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

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::Dword(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::Dword(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::Dword {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Dword(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Dword(slice))
    }
}
// SAFETY: `repr(transparent)` over one `AtomicU32`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::Float {
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

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::Float(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::Float(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::Float {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Float(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Float(slice))
    }
}
// SAFETY: `repr(transparent)` over one `AtomicU32`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::Int {
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

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::Int(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::Int(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::Int {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Int(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Int(slice))
    }
}
// SAFETY: `repr(transparent)` over one `AtomicU32`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::Flag {
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

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::Flag(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::Flag(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::Flag {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Flag(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Flag(slice))
    }
}
// SAFETY: `repr(transparent)` over one `Cell<u32>`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::PlainDword {
    fn copy_from(&self, other: &Self) {
        crate::memory::session::plain_written(self.cell());
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u32, now.cell().get(), start.cell().get()) {
            crate::memory::session::plain_written(self.cell());
            self.cell().set(v);
        }
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::PlainDword(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::PlainDword(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::PlainDword {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainDword(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainDword(slice))
    }
}
// SAFETY: `repr(transparent)` over one `Cell<u32>`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::PlainFloat {
    fn copy_from(&self, other: &Self) {
        crate::memory::session::plain_written(self.cell());
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u32, now.cell().get(), start.cell().get()) {
            crate::memory::session::plain_written(self.cell());
            self.cell().set(v);
        }
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::PlainFloat(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::PlainFloat(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::PlainFloat {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainFloat(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainFloat(slice))
    }
}
// SAFETY: `repr(transparent)` over one `Cell<u32>`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::PlainFlag {
    fn copy_from(&self, other: &Self) {
        crate::memory::session::plain_written(self.cell());
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u32, now.cell().get(), start.cell().get()) {
            crate::memory::session::plain_written(self.cell());
            self.cell().set(v);
        }
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::PlainFlag(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::PlainFlag(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::PlainFlag {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainFlag(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainFlag(slice))
    }
}

// SAFETY: `repr(transparent)` over one `Cell<u32>`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::PlainInt {
    fn copy_from(&self, other: &Self) {
        crate::memory::session::plain_written(self.cell());
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u32, now.cell().get(), start.cell().get()) {
            crate::memory::session::plain_written(self.cell());
            self.cell().set(v);
        }
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::PlainInt(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::PlainInt(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::PlainInt {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainInt(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainInt(slice))
    }
}

// SAFETY: `repr(transparent)` over one `AtomicU16`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::Word {
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

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::Word(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::Word(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::Word {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Word(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Word(slice))
    }
}
// SAFETY: `repr(transparent)` over one `AtomicU8`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::Byte {
    fn copy_from(&self, other: &Self) {
        self.atomic().store(other.atomic().load(Relaxed), Relaxed);
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        let a = self.atomic();
        if let Some(v) = merged!(u8, now.atomic().load(Relaxed), start.atomic().load(Relaxed)) {
            a.store(v, Relaxed);
        }
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::Byte(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::Byte(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::Byte {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Byte(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::Byte(slice))
    }
}
// SAFETY: `repr(transparent)` over `[AtomicU8; N]`: every byte is an atomic, and zero is a
// valid value.
unsafe impl<const N: usize> Cells for crate::memory::session::Bytes<N> {
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

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::Bytes(self.atomics()), visitor);
    }
}
// SAFETY: `repr(transparent)` over `[AtomicU8; N]`: no padding, any bits a value; all of it
// one run of byte atomics.
unsafe impl<const N: usize> ByteView for crate::memory::session::Bytes<N> {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Bytes(self.atomics()))
    }
}
// SAFETY: `repr(transparent)` over an `UnsafeCell` of a plain record: every byte is inside the
// cell, and zero is a valid value (any bytes make a `Plain`). Its runs take the widths its
// `get` / `set` take (`Shared::WORDS`): a 4-aligned record's whole dwords one run of
// `AtomicU32`s, the bytes past them `AtomicU8`s; a packed record one run of shared bytes.
unsafe impl<T: Plain> Cells for crate::memory::session::Shared<T> {
    fn copy_from(&self, other: &Self) {
        copy_by_runs(self, other);
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        merge_by_runs(self, now, start);
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_by_runs(self, visitor);
    }
}
// SAFETY: `repr(transparent)` over an `UnsafeCell<T>`, `T: Plain`: no padding, any bits a value;
// the runs are the ones `Shared::cell_run` hands out.
unsafe impl<T: Plain> ByteView for crate::memory::session::Shared<T> {
    fn cell_run(&self, offset: usize) -> (usize, CellRun<'_>) {
        crate::memory::session::Shared::cell_run(self, offset)
    }
}
// SAFETY: `repr(transparent)` over `[AtomicU8; 4]`: every byte is an atomic, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::Loose {
    fn copy_from(&self, other: &Self) {
        copy_by_runs(self, other);
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        merge_by_runs(self, now, start);
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_by_runs(self, visitor);
    }
}
// SAFETY: `repr(transparent)` over `[AtomicU8; 4]`: no padding, any bits a value; all of it
// one run of byte atomics.
unsafe impl ByteView for crate::memory::session::Loose {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::Bytes(self.atomics()))
    }
}
// SAFETY: `repr(transparent)` over one `Cell<u16>`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::PlainWord {
    fn copy_from(&self, other: &Self) {
        crate::memory::session::plain_written(self.cell());
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u16, now.cell().get(), start.cell().get()) {
            crate::memory::session::plain_written(self.cell());
            self.cell().set(v);
        }
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::PlainWord(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::PlainWord(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::PlainWord {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainWord(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainWord(slice))
    }
}
// SAFETY: `repr(transparent)` over one `Cell<u8>`: every byte is inside it, and zero is a
// valid value.
unsafe impl Cells for crate::memory::session::PlainByte {
    fn copy_from(&self, other: &Self) {
        crate::memory::session::plain_written(self.cell());
        self.cell().set(other.cell().get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        if let Some(v) = merged!(u8, now.cell().get(), start.cell().get()) {
            crate::memory::session::plain_written(self.cell());
            self.cell().set(v);
        }
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::PlainByte(std::slice::from_ref(self)), visitor);
    }

    fn visit_slice(slice: &[Self], visitor: &mut dyn CellVisitor) -> bool {
        visit_cells_slice(CellRun::PlainByte(slice), visitor)
    }
}
// SAFETY: `repr(transparent)` over one cell of an integer: no padding, any bits a value; the
// one run is the cell itself.
unsafe impl ByteView for crate::memory::session::PlainByte {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainByte(std::slice::from_ref(self)))
    }

    fn slice_run(slice: &[Self]) -> Option<CellRun<'_>> {
        Some(CellRun::PlainByte(slice))
    }
}
// SAFETY: `repr(transparent)` over `[Cell<u8>; N]`: every byte is inside a cell, and zero is a
// valid value.
unsafe impl<const N: usize> Cells for crate::memory::session::PlainBytes<N> {
    fn copy_from(&self, other: &Self) {
        for (to, from) in self.cells().iter().zip(other.cells()) {
            crate::memory::session::plain_written(to);
            to.set(from.get());
        }
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        for ((to, now), start) in self.cells().iter().zip(now.cells()).zip(start.cells()) {
            let v = now.get();
            if v != start.get() {
                crate::memory::session::plain_written(to);
                to.set(v);
            }
        }
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visit_run(CellRun::PlainBytes(self.cells()), visitor);
    }
}
// SAFETY: `repr(transparent)` over `[Cell<u8>; N]`: no padding, any bits a value; all of it
// one run of byte cells.
unsafe impl<const N: usize> ByteView for crate::memory::session::PlainBytes<N> {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainBytes(self.cells()))
    }
}
// SAFETY: `repr(transparent)` over one `Cell<[u32; N]>`: every byte is inside the cell, and
// zero is a valid value.
unsafe impl<const N: usize> Cells for crate::memory::plain_floats::PlainFloats<N> {
    fn copy_from(&self, other: &Self) {
        self.set_bits(other.bits());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        let (now, start) = (now.bits(), start.bits());
        for n in 0..N {
            if now[n] != start[n] {
                self.set_bits_at(n, now[n]);
            }
        }
    }

    /// The row as an array of one kind of cell is visited: one run, element 0.
    fn visit(&self, visitor: &mut dyn CellVisitor) {
        visitor.index(0);
        if N > 0 {
            visitor.run(CellRun::PlainWords(self.cells()));
        }
        visitor.leave();
    }
}

// SAFETY: `repr(transparent)` over a `Cell<[u32; N]>`: no padding, any bits a value; all of it one
// run of word cells.
unsafe impl<const N: usize> ByteView for crate::memory::plain_floats::PlainFloats<N> {
    fn cell_run(&self, _: usize) -> (usize, CellRun<'_>) {
        (0, CellRun::PlainWords(self.cells()))
    }
}
// SAFETY: `N` of a type of cells one after another (an array has no padding between
// elements): every value byte is in a cell of an element, and the all-zero elements are valid.
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

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        if N == 0 || T::visit_slice(self.as_slice(), visitor) {
            return;
        }
        for (i, element) in self.iter().enumerate() {
            visitor.index(i);
            element.visit(visitor);
            visitor.leave();
        }
    }
}
// SAFETY: `N` records of fixed layout one after another, an array's layout: no padding (the
// elements have none, nor does an array between them), any bits a value.
unsafe impl<T: ByteView, const N: usize> ByteView for [T; N] {
    #[cfg(test)]
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

// SAFETY: one `T` (`repr(transparent)`), whose cells are handed out only on the game's thread
// (`GameThread::get` asserts it in debug builds).
unsafe impl<T: Cells> Cells for crate::memory::session::GameThread<T> {
    fn copy_from(&self, other: &Self) {
        self.get().copy_from(other.get());
    }

    fn merge_changes(&self, now: &Self, start: &Self) {
        self.get().merge_changes(now.get(), start.get());
    }

    fn visit(&self, visitor: &mut dyn CellVisitor) {
        self.get().visit(visitor);
    }
}
// SAFETY: one `T` (`repr(transparent)`), whose cells are handed out only on the game's thread.
unsafe impl<T: ByteView> ByteView for crate::memory::session::GameThread<T> {
    #[cfg(test)]
    fn field_path(&self, offset: usize, path: &mut String) -> &'static str {
        self.get().field_path(offset, path)
    }

    fn cell_run(&self, offset: usize) -> (usize, CellRun<'_>) {
        self.get().cell_run(offset)
    }
}

impl<T: ZeroCells> ZeroCells for crate::memory::session::GameThread<T> {
    const ZERO: Self = crate::memory::session::GameThread::new(T::ZERO);
}

/// A cell or record of cells whose every byte is zero, as a constant: `#[derive(ZeroCells)]`
/// makes it for a record whose fields all have one, so a store's `new()` can name only the
/// fields that start non-zero (`Record { count: Dword::new(3), ..ZeroCells::ZERO }`).
#[allow(
    dead_code,
    reason = "no store builds on it yet: the lanes that drop gap fields will"
)]
pub trait ZeroCells: Sized {
    const ZERO: Self;
}

macro_rules! zero_cells {
    ($($t:ty = $zero:expr),* $(,)?) => { $(
        impl ZeroCells for $t {
            const ZERO: Self = $zero;
        }
    )* };
}
zero_cells!(
    crate::memory::session::Dword = crate::memory::session::Dword::new(0),
    crate::memory::session::Float = crate::memory::session::Float::new(0.0),
    crate::memory::session::Int = crate::memory::session::Int::new(0),
    crate::memory::session::Flag = crate::memory::session::Flag::new(false),
    crate::memory::session::Word = crate::memory::session::Word::new(0),
    crate::memory::session::Byte = crate::memory::session::Byte::new(0),
    crate::memory::session::Loose = crate::memory::session::Loose::new(0),
    crate::memory::session::PlainDword = crate::memory::session::PlainDword::new(0),
    crate::memory::session::PlainFloat = crate::memory::session::PlainFloat::new(0.0),
    crate::memory::session::PlainFlag = crate::memory::session::PlainFlag::new(false),
    crate::memory::session::PlainInt = crate::memory::session::PlainInt::new(0),
    crate::memory::session::PlainWord = crate::memory::session::PlainWord::new(0),
    crate::memory::session::PlainByte = crate::memory::session::PlainByte::new(0),
);
impl<const N: usize> ZeroCells for crate::memory::session::Bytes<N> {
    const ZERO: Self = crate::memory::session::Bytes::new([0; N]);
}
impl<const N: usize> ZeroCells for crate::memory::session::PlainBytes<N> {
    const ZERO: Self = crate::memory::session::PlainBytes::new([0; N]);
}
impl<T: ZeroCells, const N: usize> ZeroCells for [T; N] {
    const ZERO: Self = [const { T::ZERO }; N];
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

/// `#[repr(C)]` records of integers, their sizes the sums of their fields (no padding): each
/// pair is a type and its size in bytes. Used beside the record's definition.
macro_rules! plain_record {
    ($($t:ty = $size:expr),* $(,)?) => { $(
        const _: () = assert!(size_of::<$t>() == $size, "padding in a plain record");
        // SAFETY: integers and arrays of them only (each record's fields, read at the type), so
        // any bits are a value; no padding (asserted above).
        #[allow(unsafe_code)]
        unsafe impl $crate::memory::cells::Plain for $t {}
    )* };
}
pub(crate) use plain_record;

plain_record!(
    rc2k_core::strings::text::Font = 4 + 7 * 4,
    rc2k_core::memory::List = 4 * 4,
    rc2k_core::gfx::pixels::PixelFormat = 5 * 4,
    rc2k_core::audio::streams::AudioStream = 0x2C,
    rc2k_core::audio::modes::AudioOptions = 0x34,
    rc2k_core::audio::voices::SampleRecord = 0x40,
    rc2k_core::audio::voices::Voice = 0x70,
    rc2k_core::audio::voices::MixerChannel = 0x58,
    rc2k_core::screens::car_pages::CarDescription = 0x78,
    rc2k_core::menu::player::Player = 0x28C,
);

/// A record whose every byte is a cell, read and written by byte offset through its own cells
/// (`load_cells`, `store_cells`): the tests' reference for the typed reads.
#[cfg(test)]
pub trait Store {
    fn load(&self, offset: usize, to: &mut [u8]) -> Result<(), BadInput>;
    fn store(&self, offset: usize, from: &[u8]) -> Result<(), BadInput>;
}

#[cfg(test)]
impl<T: ByteView> Store for T {
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
        CellRef::Plain32(c) => {
            crate::memory::session::plain_written(c);
            c.set(u32::from_ne_bytes(merge(c.get().to_ne_bytes())));
        }
        CellRef::Plain8(c) => {
            crate::memory::session::plain_written(c);
            c.set(new[0]);
        }
        CellRef::Plain16(c) => {
            crate::memory::session::plain_written(c);
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
/// width (a run of `Bytes` as `memory::shared_memory` reads it).
pub fn load_cells<T: ByteView>(record: &T, offset: usize, to: &mut [u8]) -> Result<(), BadInput> {
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
pub fn store_cells<T: ByteView>(record: &T, offset: usize, from: &[u8]) -> Result<(), BadInput> {
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
    for (b, a) in to.iter_mut().zip(run) {
        *b = a.load(std::sync::atomic::Ordering::Relaxed);
    }
}

/// A run of byte atomics written.
fn store_bytes(run: &[AtomicU8], from: &[u8]) {
    for (a, &b) in run.iter().zip(from) {
        a.store(b, std::sync::atomic::Ordering::Relaxed);
    }
}

/// A record of cells with every byte zero, as a constant (a store's `new()`).
pub const fn zeroed_cells<T: Cells>() -> T {
    // SAFETY: `T` is made of cells over plain integers (`Cells`), so the all-zero value is
    // valid.
    unsafe { core::mem::MaybeUninit::<T>::zeroed().assume_init() }
}

/// `zeroed_cells` on the heap, for records too large for a thread's stack (stores of
/// megabytes; the tests' copies).
#[cfg(test)]
pub fn boxed_zeroed_cells<T: Cells>() -> Box<T> {
    // SAFETY: as `zeroed_cells`: `T` is made of cells over plain integers (`Cells`), so the
    // all-zero value is valid.
    unsafe { Box::<T>::new_zeroed().assume_init() }
}

/// The `T` at byte `offset` of `store` (any alignment; a run past the store is refused).
#[cfg(test)]
pub fn load_value<T: Plain>(store: &dyn Store, offset: usize) -> Result<T, BadInput> {
    let mut value = zeroed::<T>();
    store.load(offset, bytes_mut(&mut value))?;
    Ok(value)
}

/// `value` written at byte `offset` of `store` (a run past the store is refused).
#[cfg(test)]
pub fn store_value<T: Plain>(store: &dyn Store, offset: usize, value: T) -> Result<(), BadInput> {
    store.store(offset, bytes_of(&value))
}

#[cfg(test)]
mod tests;
