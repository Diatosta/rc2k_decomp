//! The differential tests of the marks' converted x87 blocks: each block's original assembly
//! (kept in the file's `asm_reference`) and its plain Rust version run over the same random
//! memory, at the default control word and at chop, and every place either one writes must
//! hold the same bits afterwards.

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

/// The race block's bytes at RVA `off`.
fn block_get(off: usize, len: usize) -> Vec<u8> {
    crate::pointers::GamePtr::new(crate::framework::game_addr(off))
        .read_bytes(len, "a race block place")
        .unwrap()
}

fn block_put(off: usize, bytes: &[u8]) {
    crate::pointers::GamePtr::new(crate::framework::game_addr(off))
        .write_bytes(bytes, "a race block place")
        .unwrap();
}

/// The view window's fields the assembly reads in the scene draw store (IDA 0x623F80 ..) and the
/// race view (`view_depth`, 0x63DC54), and their twins in the front end's window
/// (`scene::view_window`), which the Rust versions read: (RVA, length, twin's game
/// address).
fn window_twins() -> Vec<(usize, usize, u32)> {
    use crate::race::view_store::RaceView;
    use crate::scene::draw_store::SceneDraw;
    use core::mem::{offset_of, size_of_val};
    let w = &crate::scene::polygon_work::front_end().window;
    let twin =
        |ida: usize, len: usize, cell: *const u8| (ida, len, crate::pointers::game_address(cell));
    macro_rules! scene_field {
        ($f:ident) => {
            twin(
                0x223F80 + offset_of!(SceneDraw, $f),
                size_of_val(&w.$f),
                core::ptr::from_ref(&w.$f).cast(),
            )
        };
    }
    vec![
        scene_field!(view_offset),
        scene_field!(view_zoom),
        scene_field!(unknown_104),
        scene_field!(view_scale),
        scene_field!(view_size),
        scene_field!(view_middle),
        scene_field!(view_centre),
        scene_field!(farthest),
        scene_field!(one_over_farthest),
        scene_field!(farthest_scaled),
        scene_field!(clip_edges),
        scene_field!(clip_screen),
        scene_field!(view_offset_float),
        scene_field!(faces_done),
        scene_field!(depth_divisor),
        scene_field!(clip_farthest),
        twin(
            0x23D9B8 + offset_of!(RaceView, view_depth),
            size_of_val(&w.view_depth),
            core::ptr::from_ref(&w.view_depth).cast(),
        ),
    ]
}

/// Each listed write of the race block operands: its place and kind, and its bytes now.
#[allow(clippy::type_complexity)] // reason: a test's one-off list
fn block_outputs(spec: &Spec) -> Vec<((usize, Kind), Vec<u8>)> {
    let mut out = Vec::new();
    for op in spec.operands {
        if let Op::RaceBlock { writes, .. } = op {
            for &(off, kind) in *writes {
                out.push(((off, kind), block_get(off, SIZE(kind))));
            }
        }
    }
    out
}

/// The race block's listed places as they were before a test (put back when it ends).
struct PutBack(Vec<(usize, Vec<u8>)>);

/// The front end's window as it was before a test (put back when it ends).
struct WindowBack(crate::scene::view_window::ViewWindow);

impl Drop for WindowBack {
    fn drop(&mut self) {
        crate::pointers::Cells::copy_from(&crate::scene::polygon_work::front_end().window, &self.0);
    }
}

impl Drop for PutBack {
    fn drop(&mut self) {
        for (off, b) in self.0.iter().rev() {
            block_put(*off, b);
        }
    }
}

/// Runs `reference` and `plain` (each given the operands: base addresses and indexes)
/// over the same random memory, several thousand times at each of two control words.
pub fn differential<R: PartialEq + std::fmt::Debug>(
    spec: &Spec,
    reference: impl Fn(&[usize]) -> R,
    plain: impl Fn(&[usize]) -> R,
) {
    // The tests write the control word and the statics: one at a time, with every test of
    // the statics (`store::test_lock`).
    let _guard = crate::store::test_lock();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut failures = 0;
    let mut saved = Vec::new();
    for op in spec.operands {
        if let Op::RaceBlock { reads, writes, .. } = op {
            for &(off, kind) in reads.iter().chain(writes.iter()) {
                saved.push((off, block_get(off, SIZE(kind))));
            }
        }
    }
    let _put_back = PutBack(saved);
    // The front end's window, which the laid inputs' twins overwrite, put back at the end.
    let window = crate::scene::view_window::ViewWindow::new();
    crate::pointers::Cells::copy_from(&window, &crate::scene::polygon_work::front_end().window);
    let _window_back = WindowBack(window);
    // The race block's places the reference reads or writes (IDA's addresses), laid into
    // their old places around it (`race::store::OldPlaces`).
    let old_places: Vec<usize> = spec
        .operands
        .iter()
        .filter_map(|op| match op {
            Op::RaceBlock { reads, writes, .. } => Some(reads.iter().chain(writes.iter())),
            _ => None,
        })
        .flatten()
        .map(|&(off, _)| off + 0x400000)
        .collect();
    for cw in [0x027F, 0x0E7F] {
        x87::set_control_word(cw);
        for case in 0..3000 {
            let mut refs: Vec<Buf> = Vec::new();
            let mut values = Vec::new();
            for op in spec.operands {
                match op {
                    Op::Base { lo, hi, reads, .. } => {
                        let mut buf = Buf::new(hi - lo);
                        fill(&mut buf, *lo, reads, &mut rng, spec.sincos);
                        refs.push(buf);
                        values.push(0);
                    }
                    Op::RaceBlock { .. } => {
                        refs.push(Buf::new(0));
                        values.push(crate::race::store::asm_base() as usize);
                    }
                    Op::Index(v) => {
                        refs.push(Buf::new(0));
                        values.push(v[rng.below(v.len() as u64) as usize]);
                    }
                    Op::Float => {
                        refs.push(Buf::new(0));
                        values.push(random_float(&mut rng, spec.sincos).to_bits() as usize);
                    }
                }
            }
            let mut pls = refs.clone();
            let mut ref_vals = values.clone();
            let mut pl_vals = values;
            for (j, op) in spec.operands.iter().enumerate() {
                if let Op::Base { lo, .. } = op {
                    ref_vals[j] = (crate::pointers::game_address(refs[j].as_ptr())
                        .wrapping_sub((*lo) as u32)) as usize;
                    pl_vals[j] = (crate::pointers::game_address(pls[j].as_ptr())
                        .wrapping_sub((*lo) as u32)) as usize;
                }
            }
            // The buffers lent while the versions run: their operands are checked as game memory.
            let _lent: Vec<crate::pointers::Lent> = refs
                .iter()
                .chain(pls.iter())
                .filter(|b| !b.is_empty())
                .map(|b| crate::pointers::Lent::new(b))
                .collect();
            // the race block's places: random inputs, the outputs marked, each run from them
            let mut block_in: Vec<(usize, Vec<u8>)> = Vec::new();
            for op in spec.operands {
                if let Op::RaceBlock { reads, writes, .. } = op {
                    for &(off, kind) in *reads {
                        let mut b = vec![0u8; SIZE(kind)];
                        fill(&mut b, off, &[(off, kind)], &mut rng, spec.sincos);
                        block_in.push((off, b));
                    }
                    for &(off, kind) in *writes {
                        block_in.push((off, vec![0xAA; SIZE(kind)]));
                    }
                }
            }
            let twins = window_twins();
            let lay = || {
                for (off, b) in &block_in {
                    block_put(*off, b);
                }
                // What the assembly reads of the window, the Rust versions read in its twin.
                for &(ida, len, twin) in &twins {
                    let bytes = block_get(ida, len);
                    crate::pointers::GamePtr::new(twin)
                        .write_bytes(&bytes, "the front end's window")
                        .unwrap();
                }
            };
            lay();
            let got_ref = crate::race::store::in_old_places(&old_places, || reference(&ref_vals));
            let ref_out = block_outputs(spec);
            lay();
            let got_plain = plain(&pl_vals);
            let plain_out = block_outputs(spec);
            for ((off, kind), a) in &ref_out {
                let b = &plain_out
                    .iter()
                    .find(|(p, _)| p == &(*off, *kind))
                    .unwrap()
                    .1;
                if !same(*kind, a, b, spec.sincos) {
                    failures += 1;
                    if failures < 10 {
                        eprintln!(
                            "case {case} cw {cw:#X} race block +{off:#X} {kind:?}: reference {a:02X?} plain {b:02X?}"
                        );
                    }
                }
            }
            if got_ref != got_plain {
                failures += 1;
                if failures < 10 {
                    eprintln!("case {case} cw {cw:#X}: returned {got_ref:?} and {got_plain:?}");
                }
            }
            for (j, op) in spec.operands.iter().enumerate() {
                let Op::Base { lo, writes, .. } = op else {
                    continue;
                };
                let (a, b) = (&mut refs[j], &mut pls[j]);
                for &(off, kind) in *writes {
                    let (at, n) = (off - lo, SIZE(kind));
                    if !same(kind, &a[at..at + n], &b[at..at + n], spec.sincos) {
                        failures += 1;
                        if failures < 10 {
                            eprintln!(
                                "case {case} cw {cw:#X} operand {j} +{off:#X} {kind:?}: reference {:02X?} plain {:02X?}",
                                &a[at..at + n],
                                &b[at..at + n]
                            );
                        }
                    }
                    // equal from here on, so that what is left is the unlisted bytes
                    let copy = b[at..at + n].to_vec();
                    a[at..at + n].copy_from_slice(&copy);
                }
                if a != b {
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
    assert_eq!(failures, 0, "{failures} differences");
}
