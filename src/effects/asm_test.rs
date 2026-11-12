//! The tests of the marks' converted x87 blocks: each block's plain Rust version runs over
//! random memory, at the default control word and at chop, and every place the original's
//! assembly wrote (recorded once on i686, `test_support::vectors`) must hold the same bits
//! afterwards.

use crate::effects::store::START;
use crate::math::x87;
use crate::math::x87_float as bh;

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
    /// The race block itself (`race::store::asm_base()`, the offsets RVAs): the plain
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
        unsafe { std::slice::from_raw_parts(self.0.as_ptr().cast::<u8>(), self.1) }
    }
}

impl std::ops::DerefMut for Buf {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.0.as_mut_ptr().cast::<u8>(), self.1) }
    }
}

impl PartialEq for Buf {
    fn eq(&self, other: &Buf) -> bool {
        **self == **other
    }
}

/// Runs `f` on a typed variable block that lives at IDA address `start` in a test buffer
/// standing for the marks' block at `mk` (IDA 0x764F40): the block is copied out (its image
/// address need not be aligned for its cells) and put back afterwards. The buffer must
/// hold the whole block.
pub fn with_copy<T, R>(mk: usize, start: usize, f: impl FnOnce(&T) -> R) -> R {
    crate::effects::typed_sync::fits_buffer(
        mk.wrapping_add(start).wrapping_sub(START),
        size_of::<T>(),
    );
    let p = crate::pointers::host_at(mk.wrapping_add(start).wrapping_sub(START)).cast::<T>();
    // The copy on the heap: some blocks are near 2 MB, a test thread's whole stack (copied
    // by value there they overflowed it, and Wine hung on the fault).
    unsafe {
        let mut local = Box::<T>::new_uninit();
        core::ptr::copy_nonoverlapping(
            p.cast::<u8>(),
            local.as_mut_ptr().cast::<u8>(),
            size_of::<T>(),
        );
        let local = local.assume_init();
        let out = f(&local);
        core::ptr::copy_nonoverlapping(
            (&raw const *local).cast::<u8>(),
            p.cast::<u8>(),
            size_of::<T>(),
        );
        out
    }
}

/// Runs `f` on the marks' settings record at the test address `base` (an operand with
/// `lo: 0, hi: 0x2768`), as `with_copy`.
pub fn with_settings<R>(
    base: usize,
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
/// the original's instructions made, recorded under `name` (see `test_support::vectors`):
/// every listed write, the buffers' other bytes, the returned value's printed form.
pub fn recorded<R: std::fmt::Debug>(name: &str, spec: &Spec, plain: impl Fn(&[usize]) -> R) {
    use crate::test_support::vectors::Recorder;
    run_recorded(Recorder::replay(name), spec, &plain);
}

fn run_recorded<R: std::fmt::Debug>(
    mut file: crate::test_support::vectors::Recorder,
    spec: &Spec,
    plain: &dyn Fn(&[usize]) -> R,
) {
    assert!(
        !spec
            .operands
            .iter()
            .any(|op| matches!(op, Op::RaceBlock { .. })),
        "a recorded block reads only its test buffers"
    );
    let _guard = crate::store::test_lock();
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
            // The operands' values: each buffer by game address, less its first offset.
            let run = |bufs: &[Buf]| -> Vec<usize> {
                spec.operands
                    .iter()
                    .enumerate()
                    .map(|(j, op)| match op {
                        Op::Base { lo, .. } => {
                            (crate::pointers::game_address(bufs[j].as_ptr())
                                .wrapping_sub(*lo as u32)) as usize
                        }
                        _ => values[j],
                    })
                    .collect()
            };
            // Replaying: the results are the file's.
            let expected = file.results(&before, || unreachable!("vectors are replayed"));
            let returned = file.result_bytes(|| unreachable!("vectors are replayed"));
            let pls = start.clone();
            let got = {
                let _lent: Vec<crate::pointers::Lent> = pls
                    .iter()
                    .filter(|b| !b.is_empty())
                    .map(|b| crate::pointers::Lent::new(b))
                    .collect();
                format!("{:?}", plain(&run(&pls)))
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
    let w = &crate::scene::polygon_work::front_end().window;
    let m = &crate::race::owner::race().frame.matrices;
    match off {
        0x224088 => &w.view_scale,
        0x22409C => &w.view_centre[0],
        0x2240A0 => &w.view_centre[1],
        0x2241C8 => &w.farthest,
        0x31C814 => &crate::effects::owner::effects().setup.work.fpu_status,
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
/// original's instructions made, recorded under `name` (see `test_support::vectors`): every
/// listed write (the buffers' and the race block's) and the returned value's printed form.
/// The inputs are laid into the typed cells of the places (`race_cell`); the bytes outside
/// the listed writes must be left as they were.
pub fn recorded_writes<R: std::fmt::Debug>(name: &str, spec: &Spec, plain: impl Fn(&[usize]) -> R) {
    use crate::test_support::vectors::Recorder;
    run_writes(Recorder::replay(name), spec, &plain);
}

fn run_writes<R: std::fmt::Debug>(
    mut file: crate::test_support::vectors::Recorder,
    spec: &Spec,
    plain: &dyn Fn(&[usize]) -> R,
) {
    let _guard = crate::store::test_lock();
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
            let run = |bufs: &[Buf]| -> Vec<usize> {
                spec.operands
                    .iter()
                    .enumerate()
                    .map(|(j, op)| match op {
                        Op::Base { lo, .. } => {
                            (crate::pointers::game_address(bufs[j].as_ptr())
                                .wrapping_sub(*lo as u32)) as usize
                        }
                        _ => values[j],
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
            let pls = start.clone();
            let got = {
                let _lent: Vec<crate::pointers::Lent> = pls
                    .iter()
                    .filter(|b| !b.is_empty())
                    .map(|b| crate::pointers::Lent::new(b))
                    .collect();
                format!("{:?}", plain(&run(&pls)))
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
