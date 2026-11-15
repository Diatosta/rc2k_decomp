//! The tests of the marks' converted x87 blocks: each block's plain Rust version runs over
//! random memory, at the default control word and at chop, and every place the recorded
//! x87 assembly wrote (recorded once on i686, `test_support::vectors`) must hold the same
//! bits afterwards.

use crate::math::x87;
use crate::math::x87_float as bh;

/// Where the recorded layout of the effects' variables begins (the test buffers stand for
/// the block from here; the x87 reference tests address operands by that layout).
pub const START: usize = 0x764F40;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    F32,
    I32,
    F80,
    Status,
    U32,
}
pub use Kind::*;

pub enum Op {
    /// A base address: the bytes `lo .. hi` of its region are read and written at the
    /// given offsets.
    Base {
        lo: usize,
        hi: usize,
        reads: &'static [(usize, Kind)],
        writes: &'static [(usize, Kind)],
    },
    /// The race block itself (its places named by offset): the plain
    /// versions read it through the typed stores. Only the listed places are filled,
    /// compared and put back.
    RaceBlock {
        reads: &'static [(usize, Kind)],
        writes: &'static [(usize, Kind)],
    },
    /// A register that adds to an address: one of these values.
    Index(&'static [usize]),
    /// A float (the bits of a random float) the block starts with on the x87 stack.
    Float,
}

pub struct Spec {
    pub operands: &'static [Op],
    /// The block has `fsin` / `fcos` (libm's, one float ulp off at most).
    pub sincos: bool,
}

/// One operand of a recorded block, as the plain version gets it: a test buffer addressed by
/// the coordinates the recorded instructions used (the buffer's first byte stands for
/// coordinate `lo`), or a value (an index, a float's bits). Adding to a buffer operand moves
/// its coordinate, as adding to the address did; every access is checked against the buffer.
#[derive(Clone, Copy, Debug)]
pub struct Operand {
    /// The host address of coordinate 0 (wrapping).
    base: usize,
    start: usize,
    end: usize,
    value: usize,
}

impl Operand {
    fn buffer(bytes: *mut u8, len: usize, lo: usize) -> Operand {
        let start = bytes as usize;
        Operand {
            base: start.wrapping_sub(lo),
            start,
            end: start + len,
            value: 0,
        }
    }

    fn value(value: usize) -> Operand {
        Operand {
            base: 0,
            start: 0,
            end: 0,
            value,
        }
    }

    /// The operand as a number (an index or a float's bits).
    #[track_caller]
    pub fn number(self) -> usize {
        assert!(self.end == 0, "a buffer operand used as a number");
        self.value
    }

    /// The operand moved on by `k` bytes.
    pub fn wrapping_add(self, k: usize) -> Operand {
        Operand {
            base: self.base.wrapping_add(k),
            ..self
        }
    }

    /// The `len` bytes at coordinate `off` of the operand, which must lie in its buffer.
    #[track_caller]
    pub fn at(self, off: usize, len: usize) -> *mut u8 {
        let p = self.base.wrapping_add(off);
        assert!(
            p >= self.start && p.checked_add(len).is_some_and(|e| e <= self.end),
            "{len:#x} bytes at +{off:#x} do not lie in the operand's test buffer"
        );
        p as *mut u8
    }

    /// The host pointer to this coordinate, which must lie in the buffer.
    #[track_caller]
    pub fn host(self) -> *mut u8 {
        self.at(0, 1)
    }

    /// Whether the `len` bytes at `off` lie in the buffer.
    pub fn contains(self, off: usize, len: usize) -> bool {
        let p = self.base.wrapping_add(off);
        p >= self.start && p.checked_add(len).is_some_and(|e| e <= self.end)
    }

    /// The `len` bytes at `off`, the ones outside the buffer 0 (a block that reads only part
    /// of it).
    pub fn bytes_or_zero(self, off: usize, len: usize) -> Vec<u8> {
        (0..len)
            .map(|i| {
                if self.contains(off + i, 1) {
                    // SAFETY: the byte lies in the test buffer (checked just above, and by
                    // `at`), which the operand's run keeps alive.
                    unsafe { self.at(off + i, 1).read() }
                } else {
                    0
                }
            })
            .collect()
    }

    /// The `len` bytes at `off`, which must lie in the buffer.
    #[track_caller]
    pub fn read(self, off: usize, len: usize) -> Vec<u8> {
        // SAFETY: `at` checked that the bytes lie in the test buffer.
        unsafe { std::slice::from_raw_parts(self.at(off, len), len).to_vec() }
    }

    /// `bytes` at `off`.
    #[track_caller]
    pub fn put(self, off: usize, bytes: &[u8]) {
        // SAFETY: `at` checks that the `bytes.len()` bytes lie in the test buffer, which the
        // operand's run keeps alive; `bytes` is the caller's, not the buffer.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.at(off, bytes.len()), bytes.len());
        }
    }

    /// The `N` bytes at `off`.
    #[track_caller]
    pub fn bytes<const N: usize>(self, off: usize) -> [u8; N] {
        // SAFETY: `at` checks that the `N` bytes lie in the test buffer, which the operand's run
        // keeps alive; the read needs no alignment.
        unsafe { self.at(off, N).cast::<[u8; N]>().read_unaligned() }
    }
}

impl std::ops::Add<usize> for Operand {
    type Output = Operand;
    fn add(self, k: usize) -> Operand {
        self.wrapping_add(k)
    }
}

impl std::ops::Sub<usize> for Operand {
    type Output = Operand;
    fn sub(self, k: usize) -> Operand {
        self.wrapping_add(k.wrapping_neg())
    }
}

impl crate::scene::x87_mem::Base for Operand {
    #[track_caller]
    fn dword(self, off: usize) -> u32 {
        u32::from_le_bytes(self.bytes(off))
    }
    #[track_caller]
    fn set_dword(self, off: usize, value: u32) {
        self.put(off, &value.to_le_bytes());
    }
    #[track_caller]
    fn set_word(self, off: usize, value: u16) {
        self.put(off, &value.to_le_bytes());
    }
    #[track_caller]
    fn tbyte(self, off: usize) -> [u8; 10] {
        self.bytes(off)
    }
    #[track_caller]
    fn set_tbyte(self, off: usize, value: [u8; 10]) {
        self.put(off, &value);
    }
}

/// A test buffer of bytes whose first byte is 16-aligned (a typed struct is laid over a
/// block's bytes at an aligned offset).
#[derive(Clone)]
struct Buf(Vec<u128>, usize);

impl Buf {
    fn new(len: usize) -> Buf {
        Buf(vec![0; len.div_ceil(16)], len)
    }
}

impl std::ops::Deref for Buf {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        // SAFETY: the vector's `len.div_ceil(16)` `u128`s hold at least `self.1` bytes (`new`),
        // any bits make a byte, and the slice borrows `self`.
        unsafe { std::slice::from_raw_parts(self.0.as_ptr().cast::<u8>(), self.1) }
    }
}

impl std::ops::DerefMut for Buf {
    fn deref_mut(&mut self) -> &mut [u8] {
        // SAFETY: as `deref`'s, the slice borrowing `self` mutably.
        unsafe { std::slice::from_raw_parts_mut(self.0.as_mut_ptr().cast::<u8>(), self.1) }
    }
}

impl PartialEq for Buf {
    fn eq(&self, other: &Buf) -> bool {
        **self == **other
    }
}

/// Runs `f` on a record of cells that lives at offset `start` of the marks' block in the test
/// buffer `mk` (the buffer stands for the marks' block): the record's cells are made from the
/// bytes there, one after another in field order (no gaps in the recorded layout), and put back
/// afterwards. The buffer must hold the whole record.
pub fn with_copy<T: crate::cells::Cells, R>(
    mk: Operand,
    start: usize,
    f: impl FnOnce(&T) -> R,
) -> R {
    use crate::test_support::paths::{field_bytes, set_field_range};
    let at = start.wrapping_sub(START);
    // The copy on the heap: some blocks are near 2 MB, a test thread's whole stack (copied
    // by value there they overflowed it, and Wine hung on the fault).
    let local = crate::cells::boxed_zeroed_cells::<T>();
    let len = field_bytes(&*local, "").len();
    assert!(set_field_range(&*local, "", &mk.read(at, len)));
    let out = f(&local);
    mk.put(at, &field_bytes(&*local, ""));
    out
}

/// The `N` floats in a row at offset `start` of the marks' block in the test buffer `mk`, for
/// the plain-float constant tables the vectors read.
pub fn floats_at<const N: usize>(mk: Operand, start: usize) -> [f32; N] {
    let bytes = mk.read(start.wrapping_sub(START), 4 * N);
    std::array::from_fn(|i| f32::from_le_bytes(bytes[4 * i..4 * i + 4].try_into().unwrap()))
}

/// Runs `f` on the marks' settings record at the test address `base` (an operand with
/// `lo: 0, hi: 0x2768`), as `with_copy`.
pub fn with_settings<R>(
    base: Operand,
    f: impl FnOnce(&crate::race::effects_settings_store::EffectsSettings) -> R,
) -> R {
    with_copy(base, START, f)
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn random_float(r: &mut Rng, sincos: bool) -> f32 {
    let sign = if r.below(2) == 0 { 1.0 } else { -1.0 };
    let v = match r.below(12) {
        0 => 0.0,
        1 => (r.below(2001) as i64 - 1000) as f64,
        2 => (r.below(2001) as i64 - 1000) as f64 / 2.0,
        3 => rc2k_core::math::portable::exp2(r.unit() * 40.0 - 20.0),
        4 => r.unit() * 1e-6,
        5 if !sincos => r.unit() * 1e9,
        6 => r.unit(),
        7 => r.unit() * 6.3,
        _ => r.unit() * 1000.0,
    };
    (sign * v) as f32
}

fn random_int(r: &mut Rng) -> i32 {
    match r.below(6) {
        0 => 0,
        1 => r.below(65) as i32 - 32,
        2 => r.next() as i32,
        3 => r.below(0x2_0000) as i32 - 0x1_0000,
        _ => r.below(4001) as i32 - 2000,
    }
}

fn fill(buf: &mut [u8], lo: usize, reads: &[(usize, Kind)], r: &mut Rng, sincos: bool) {
    buf.fill(0xAA);
    for &(off, kind) in reads {
        let at = off - lo;
        match kind {
            F32 => {
                buf[at..at + 4].copy_from_slice(&random_float(r, sincos).to_bits().to_le_bytes());
            }
            I32 => buf[at..at + 4].copy_from_slice(&random_int(r).to_le_bytes()),
            F80 => {
                let f = f64::from(random_float(r, sincos)) * (1.0 + r.unit() * 1e-3);
                buf[at..at + 10].copy_from_slice(&bh::f80(bh::X(f)));
            }
            Status | U32 => {}
        }
    }
}

/// A float the block leaves on the x87 stack, as bits (any NaN the same).
pub fn canon(x: f64) -> u64 {
    if x.is_nan() {
        0x7FF8_0000_0000_0000
    } else {
        x.to_bits()
    }
}

fn f32_nan(b: u32) -> bool {
    b & 0x7F80_0000 == 0x7F80_0000 && b & 0x007F_FFFF != 0
}

fn same(kind: Kind, a: &[u8], b: &[u8], ulp: bool) -> bool {
    match kind {
        F32 => {
            let (a, b) = (
                u32::from_le_bytes(a.try_into().unwrap()),
                u32::from_le_bytes(b.try_into().unwrap()),
            );
            if a == b || (f32_nan(a) && f32_nan(b)) {
                return true;
            }
            // one float step, for libm's sine and cosine
            ulp && a.abs_diff(b) <= 1 && (a >> 31) == (b >> 31) && !f32_nan(a) && !f32_nan(b)
                || ulp && (a & 0x7FFF_FFFF) == 0 && (b & 0x7FFF_FFFF) == 0
        }
        I32 | U32 => a == b,
        F80 => {
            if a == b {
                return true;
            }
            let nan = |v: &[u8]| {
                let e = u16::from_le_bytes([v[8], v[9]]) & 0x7FFF;
                let m = u64::from_le_bytes(v[..8].try_into().unwrap());
                e == 0x7FFF && m << 1 != 0
            };
            nan(a) && nan(b)
        }
        Status => {
            (u16::from_le_bytes(a.try_into().unwrap()) & 0x4500)
                == (u16::from_le_bytes(b.try_into().unwrap()) & 0x4500)
        }
    }
}

const SIZE: fn(Kind) -> usize = |k| match k {
    F32 | I32 | U32 => 4,
    F80 => 10,
    Status => 2,
};

/// A block over test buffers only (no race block operand): `plain` compared with the results
/// the recorded instructions made, recorded under `name` (see `test_support::vectors`):
/// every listed write, the buffers' other bytes, the returned value's printed form.
pub fn recorded<R: std::fmt::Debug>(name: &str, spec: &Spec, plain: impl Fn(&[Operand]) -> R) {
    use crate::test_support::vectors::Recorder;
    run_recorded(Recorder::replay(name), spec, &plain);
}

fn run_recorded<R: std::fmt::Debug>(
    mut file: crate::test_support::vectors::Recorder,
    spec: &Spec,
    plain: &dyn Fn(&[Operand]) -> R,
) {
    assert!(
        !spec
            .operands
            .iter()
            .any(|op| matches!(op, Op::RaceBlock { .. })),
        "a recorded block reads only its test buffers"
    );
    // No `test_support::test_lock`: the run reaches no static. The control word it sets is
    // this thread's (`math::x87`'s thread-local one, and in the i686 tests the x87's own, a
    // thread's register), the operands are the run's own buffers, the file is read only, and
    // every caller's `plain` works on its operands only: checked for each
    // (`effects::{adjugate3, pong_ball_hits, run_windscreen_flakes, turns}`): x87 arithmetic
    // over the buffers' bytes and over records laid over them (`asm_layout::with_laid`, a
    // fresh copy filled from the buffer). A new caller whose `plain` reaches a static takes
    // the lock around its call.
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut failures = 0;
    for cw in [0x027F, 0x0E7F] {
        x87::set_control_word(cw);
        for case in 0..3000 {
            let mut start: Vec<Buf> = Vec::new();
            let mut values = Vec::new();
            for op in spec.operands {
                match op {
                    Op::Base { lo, hi, reads, .. } => {
                        let mut buf = Buf::new(hi - lo);
                        fill(&mut buf, *lo, reads, &mut rng, spec.sincos);
                        start.push(buf);
                        values.push(0);
                    }
                    Op::RaceBlock { .. } => unreachable!(),
                    Op::Index(v) => {
                        start.push(Buf::new(0));
                        values.push(v[rng.below(v.len() as u64) as usize]);
                    }
                    Op::Float => {
                        start.push(Buf::new(0));
                        values.push(random_float(&mut rng, spec.sincos).to_bits() as usize);
                    }
                }
            }
            let before: Vec<u8> = start.iter().flat_map(|b| b.iter().copied()).collect();
            file.input(&before);
            for v in &values {
                file.input(&(*v as u64).to_le_bytes());
            }
            // The operands' values: each buffer by host address, less its first offset.
            let run = |bufs: &mut [Buf]| -> Vec<Operand> {
                spec.operands
                    .iter()
                    .zip(bufs.iter_mut())
                    .enumerate()
                    .map(|(j, (op, buf))| match op {
                        Op::Base { lo, .. } => {
                            Operand::buffer(buf.0.as_mut_ptr().cast(), buf.1, *lo)
                        }
                        _ => Operand::value(values[j]),
                    })
                    .collect()
            };
            // Replaying: the results are the file's.
            let expected = file.results(&before, || unreachable!("vectors are replayed"));
            let returned = file.result_bytes(|| unreachable!("vectors are replayed"));
            let mut pls = start.clone();
            let got = {
                let operands = run(&mut pls);
                format!("{:?}", plain(&operands))
            };
            if got.as_bytes() != returned {
                failures += 1;
                if failures < 10 {
                    eprintln!(
                        "case {case} cw {cw:#X}: returned {} and {got}",
                        String::from_utf8_lossy(&returned)
                    );
                }
            }
            let mut at = 0;
            for (j, op) in spec.operands.iter().enumerate() {
                let len = pls[j].len();
                let mut a = expected[at..at + len].to_vec();
                at += len;
                let Op::Base { lo, writes, .. } = op else {
                    continue;
                };
                let b = &pls[j];
                for &(off, kind) in *writes {
                    let (o, n) = (off - lo, SIZE(kind));
                    if !same(kind, &a[o..o + n], &b[o..o + n], spec.sincos) {
                        failures += 1;
                        if failures < 10 {
                            eprintln!(
                                "case {case} cw {cw:#X} operand {j} +{off:#X} {kind:?}: reference {:02X?} plain {:02X?}",
                                &a[o..o + n],
                                &b[o..o + n]
                            );
                        }
                    }
                    // equal from here on, so that what is left is the unlisted bytes
                    a[o..o + n].copy_from_slice(&b[o..o + n]);
                }
                if a[..] != b[..] {
                    failures += 1;
                    if failures < 10 {
                        let k = a.iter().zip(b.iter()).position(|(x, y)| x != y).unwrap();
                        eprintln!(
                            "case {case} cw {cw:#X} operand {j} +{:#X}: unlisted byte differs {:02X} {:02X}",
                            lo + k,
                            a[k],
                            b[k]
                        );
                    }
                }
            }
        }
    }
    x87::set_control_word(0x027F);
    file.finish();
    assert_eq!(failures, 0, "{failures} differences");
}

// ---- the sparse vectors: only the listed writes are recorded ----

/// Race-block places the blocks read or write, as the typed cells that hold them (what the
/// Rust versions read), by the place's offset in the race block.
fn race_cell(off: usize) -> &'static dyn crate::session::Cell32 {
    let w = &crate::owners::Owners::the()
        .scene
        .scratch
        .front_end_polygon()
        .window;
    let m = &crate::test_support::own::race().frame.matrices.get();
    match off {
        0x224088 => &w.view_scale,
        0x22409C => &w.view_centre[0],
        0x2240A0 => &w.view_centre[1],
        0x2241C8 => &w.farthest,
        0x31C814 => &crate::test_support::own::effects().setup.work.fpu_status,
        0x1EA2CC..0x1EA30C if off.is_multiple_of(4) => &m.view_matrix[(off - 0x1EA2CC) / 4],
        0x1EA3CC..0x1EA40C if off.is_multiple_of(4) => &m.view_place[(off - 0x1EA3CC) / 4],
        _ => panic!("race block place {off:#X} has no typed cell in the effects tests"),
    }
}

fn lay_race_place(off: usize, bytes: &[u8]) {
    let c = race_cell(off);
    match bytes.len() {
        4 => c.set_bits(u32::from_le_bytes(bytes.try_into().unwrap())),
        2 => c.set_bits(
            (c.bits() & 0xFFFF_0000) | u32::from(u16::from_le_bytes(bytes.try_into().unwrap())),
        ),
        n => panic!("race block place of {n} bytes"),
    }
}

fn read_race_place(off: usize, len: usize) -> Vec<u8> {
    let bits = race_cell(off).bits();
    bits.to_le_bytes()[..len].to_vec()
}

/// What a race block operand's places must hold before and after: the cells hold what the
/// test left there, so a test puts them back (the front end's window and the matrices are
/// shared statics).
struct CellsBack(Vec<(usize, u32)>);

impl Drop for CellsBack {
    fn drop(&mut self) {
        for &(off, bits) in &self.0 {
            race_cell(off).set_bits(bits);
        }
    }
}

/// A block over test buffers and race block places: `plain` compared with the results the
/// recorded instructions made, recorded under `name` (see `test_support::vectors`): every
/// listed write (the buffers' and the race block's) and the returned value's printed form.
/// The inputs are laid into the typed cells of the places (`race_cell`); the bytes outside
/// the listed writes must be left as they were.
pub fn recorded_writes<R: std::fmt::Debug>(
    name: &str,
    spec: &Spec,
    plain: impl Fn(&[Operand]) -> R,
) {
    use crate::test_support::vectors::Recorder;
    run_writes(Recorder::replay(name), spec, &plain, true);
}

/// `recorded_writes` for a block over test buffers only whose `plain` reaches no static: run
/// without `test_support::test_lock`, as `recorded` (whose reasons hold here). Checked for
/// each caller (`effects::{pong, scale_axes}`): x87 arithmetic over the buffers' bytes and
/// over records laid over them (`asm_layout::with_laid`).
pub fn recorded_buffer_writes<R: std::fmt::Debug>(
    name: &str,
    spec: &Spec,
    plain: impl Fn(&[Operand]) -> R,
) {
    use crate::test_support::vectors::Recorder;
    assert!(
        !spec
            .operands
            .iter()
            .any(|op| matches!(op, Op::RaceBlock { .. })),
        "a block of buffer writes reads only its test buffers"
    );
    run_writes(Recorder::replay(name), spec, &plain, false);
}

/// The lock is taken unless the caller says `plain` reaches no static and no operand is in
/// the race block (`recorded_buffer_writes`).
fn run_writes<R: std::fmt::Debug>(
    mut file: crate::test_support::vectors::Recorder,
    spec: &Spec,
    plain: &dyn Fn(&[Operand]) -> R,
    reaches_statics: bool,
) {
    let _guard = reaches_statics.then(crate::test_support::test_lock);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut failures = 0;
    // The race block places any operand lists: put back when the test ends.
    let mut places: Vec<usize> = Vec::new();
    for op in spec.operands {
        if let Op::RaceBlock { reads, writes } = op {
            places.extend(reads.iter().chain(writes.iter()).map(|&(o, _)| o));
        }
    }
    let _back = CellsBack(places.iter().map(|&o| (o, race_cell(o).bits())).collect());
    for cw in [0x027F, 0x0E7F] {
        x87::set_control_word(cw);
        for case in 0..3000 {
            let mut start: Vec<Buf> = Vec::new();
            let mut values = Vec::new();
            let mut block_in: Vec<(usize, Vec<u8>)> = Vec::new();
            for op in spec.operands {
                match op {
                    Op::Base { lo, hi, reads, .. } => {
                        let mut buf = Buf::new(hi - lo);
                        fill(&mut buf, *lo, reads, &mut rng, spec.sincos);
                        start.push(buf);
                        values.push(0);
                    }
                    Op::RaceBlock { reads, writes } => {
                        start.push(Buf::new(0));
                        values.push(0);
                        for &(off, kind) in reads.iter() {
                            let mut b = vec![0u8; SIZE(kind)];
                            fill(&mut b, off, &[(off, kind)], &mut rng, spec.sincos);
                            block_in.push((off, b));
                        }
                        for &(off, kind) in writes.iter() {
                            block_in.push((off, vec![0xAA; SIZE(kind)]));
                        }
                    }
                    Op::Index(v) => {
                        start.push(Buf::new(0));
                        values.push(v[rng.below(v.len() as u64) as usize]);
                    }
                    Op::Float => {
                        start.push(Buf::new(0));
                        values.push(random_float(&mut rng, spec.sincos).to_bits() as usize);
                    }
                }
            }
            // The buffers' other bytes are all 0xAA: the places filled are the input.
            for (op, buf) in spec.operands.iter().zip(&start) {
                if let Op::Base { lo, reads, .. } = op {
                    for &(off, kind) in *reads {
                        file.input(&buf[off - lo..off - lo + SIZE(kind)]);
                    }
                }
            }
            for v in &values {
                file.input(&(*v as u64).to_le_bytes());
            }
            for (_, b) in &block_in {
                file.input(b);
            }
            // The listed writes: (operand, offset in the operand or the place, kind).
            let writes: Vec<(usize, usize, Kind)> = spec
                .operands
                .iter()
                .enumerate()
                .flat_map(|(j, op)| -> Vec<(usize, usize, Kind)> {
                    match op {
                        Op::Base { lo, writes, .. } => {
                            writes.iter().map(|&(o, k)| (j, o - lo, k)).collect()
                        }
                        Op::RaceBlock { writes, .. } => {
                            writes.iter().map(|&(o, k)| (j, o, k)).collect()
                        }
                        _ => Vec::new(),
                    }
                })
                .collect();
            // Each write's bytes before the block ran.
            let before: Vec<u8> = writes
                .iter()
                .flat_map(|&(j, o, k)| match spec.operands[j] {
                    Op::RaceBlock { .. } => vec![0xAA; SIZE(k)],
                    _ => start[j][o..o + SIZE(k)].to_vec(),
                })
                .collect();
            let run = |bufs: &mut [Buf]| -> Vec<Operand> {
                spec.operands
                    .iter()
                    .zip(bufs.iter_mut())
                    .enumerate()
                    .map(|(j, (op, buf))| match op {
                        Op::Base { lo, .. } => {
                            Operand::buffer(buf.0.as_mut_ptr().cast(), buf.1, *lo)
                        }
                        _ => Operand::value(values[j]),
                    })
                    .collect()
            };
            // Replaying: the results are the file's.
            let expected = file.results(&before, || unreachable!("vectors are replayed"));
            let returned = file.result_bytes(|| unreachable!("vectors are replayed"));
            // The Rust version over the same inputs.
            for (off, b) in &block_in {
                lay_race_place(*off, b);
            }
            let mut pls = start.clone();
            let got = {
                let operands = run(&mut pls);
                format!("{:?}", plain(&operands))
            };
            if got.as_bytes() != returned {
                failures += 1;
                if failures < 10 {
                    eprintln!(
                        "case {case} cw {cw:#X}: returned {} and {got}",
                        String::from_utf8_lossy(&returned)
                    );
                }
            }
            let mut at = 0;
            for &(j, o, k) in &writes {
                let n = SIZE(k);
                let a = &expected[at..at + n];
                at += n;
                let b = match spec.operands[j] {
                    Op::RaceBlock { .. } => read_race_place(o, n),
                    _ => pls[j][o..o + n].to_vec(),
                };
                if !same(k, a, &b, spec.sincos) {
                    failures += 1;
                    if failures < 10 {
                        eprintln!(
                            "case {case} cw {cw:#X} operand {j} +{o:#X} {k:?}: reference {a:02X?} plain {b:02X?}"
                        );
                    }
                }
                // equal from here on, so that what is left is the unlisted bytes
            }
            for (j, b) in pls.iter().enumerate() {
                let mut a = start[j].to_vec();
                for &(wj, o, k) in &writes {
                    if wj == j && !b.is_empty() {
                        a[o..o + SIZE(k)].copy_from_slice(&b[o..o + SIZE(k)]);
                    }
                }
                if a[..] != b[..] {
                    failures += 1;
                    if failures < 10 {
                        let k = a.iter().zip(b.iter()).position(|(x, y)| x != y).unwrap();
                        eprintln!(
                            "case {case} cw {cw:#X} operand {j} +{k:#X}: unlisted byte differs {:02X} {:02X}",
                            a[k], b[k]
                        );
                    }
                }
            }
        }
    }
    x87::set_control_word(0x027F);
    file.finish();
    assert_eq!(failures, 0, "{failures} differences");
}

/// The vertex float at byte `off` of the vertices (x, y, z, u or v of a vertex).
#[cfg(test)]
pub fn vertex_cell(off: usize) -> &'static crate::session::PlainFloat {
    let v = &crate::d3d::vertices_store::VERTICES.vertices[off / 32];
    match off % 32 {
        0x0 => &v.x,
        0x4 => &v.y,
        0x8 => &v.z,
        0x18 => &v.u,
        0x1C => &v.v,
        _ => panic!("not a tested vertex field"),
    }
}

/// Runs `f` with the vertex floats at `offsets` holding the test buffer's (at `at`) bytes;
/// what `f` leaves goes back to the buffer, the vertices as they were. The copies go by bits
/// on purpose: they are bit-exact whatever the float, where a float moved through the x87
/// (32-bit builds) would quiet a signalling NaN.
#[cfg(test)]
pub fn staged_vertices<R>(at: Operand, offsets: &[usize], f: impl FnOnce() -> R) -> R {
    let host = |off: usize| (at + off).at(0, 4).cast::<u32>();
    let saved: Vec<u32> = offsets.iter().map(|&o| vertex_cell(o).bits()).collect();
    for &o in offsets {
        // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
        // recorded run keeps alive; unaligned, and any bits make a `u32`.
        vertex_cell(o).set_bits(unsafe { host(o).read_unaligned() });
    }
    let out = f();
    for (&o, &old) in offsets.iter().zip(&saved) {
        // SAFETY: as above.
        unsafe { host(o).write_unaligned(vertex_cell(o).bits()) };
        vertex_cell(o).set_bits(old);
    }
    out
}
