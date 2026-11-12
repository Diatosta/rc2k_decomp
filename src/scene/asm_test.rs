//! Support for the differential tests of the blocks that were x87 assembly (each file keeps
//! the original block in its `asm_reference` module): seeded random inputs, the control words
//! the game runs under, a bit-exact comparison.

use crate::math::x87::{control_word, set_control_word};
use std::sync::MutexGuard;

/// Every test that runs assembly takes this (the tests share the x87 state and statics):
/// `crate::store::test_lock`, shared with every test of the statics.
pub fn lock() -> MutexGuard<'static, ()> {
    crate::store::test_lock()
}

pub use crate::test_support::vectors::{
    CONTROL_WORDS, Kind, Rng, close_float, put_t80, same_float,
};

struct Restore(u16);
impl Drop for Restore {
    fn drop(&mut self) {
        set_control_word(self.0);
    }
}

/// Runs `cases` random memory images through the reference and the plain version at each
/// control word and compares the images dword by dword as `kinds` say (the lengths agree).
/// `fill` writes one case's input; both versions get a copy of it and a base pointer.
pub fn differential<const N: usize>(
    name: &str,
    cases: u32,
    seed: u64,
    kinds: [Kind; N],
    fill: impl Fn(&mut Rng, &mut [u32; N]),
    reference: impl Fn(*mut u8),
    plain: impl Fn(*mut u8),
) {
    let _g = lock();
    let _restore = Restore(control_word());
    let mut rng = Rng::new(seed);
    let (mut compared, mut nans) = (0u64, 0u64);
    for &cw in &CONTROL_WORDS {
        set_control_word(cw);
        for case in 0..cases {
            let mut input = [0u32; N];
            fill(&mut rng, &mut input);
            let (mut a, mut b) = (input, input);
            reference(a.as_mut_ptr().cast());
            plain(b.as_mut_ptr().cast());
            for k in 0..N {
                if matches!(kinds[k], Kind::Float | Kind::Ulp1) {
                    compared += 1;
                    nans += u64::from(f32::from_bits(a[k]).is_nan());
                }
                let ok = match kinds[k] {
                    Kind::Exact => a[k] == b[k],
                    Kind::Float => same_float(a[k], b[k]),
                    Kind::Ulp1 => close_float(a[k], b[k], 1),
                };
                assert!(
                    ok,
                    "{name}: control word {cw:#06X}, case {case}, dword {k}: reference {:#010X} ({}), plain {:#010X} ({}); input {:08X?}",
                    a[k],
                    f32::from_bits(a[k]),
                    b[k],
                    f32::from_bits(b[k]),
                    input
                );
            }
        }
    }
    // Results that are all NaN compare equal whatever they are: the inputs must reach numbers.
    assert!(
        nans * 4 <= compared,
        "{name}: {nans} of {compared} compared dwords are NaN"
    );
}

// ---- the sparse tests: blocks that touch a few places of a large memory ----

/// A reference block: the original's instructions as a function over the operands' values
/// (addresses or indexes), the registers the block names.
macro_rules! reference {
    ($name:ident ($($arg:ident),*) $($line:literal),+ $(,)?) => {
        unsafe fn $name($($arg: usize),*) {
            unsafe {
                core::arch::asm!(
                    $($line,)+
                    $($arg = in(reg) $arg,)*
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            }
        }
    };
}
pub(crate) use reference;

/// `reference!` for a block with integer instructions that use eax.
macro_rules! reference_eax {
    ($name:ident ($($arg:ident),*) $($line:literal),+ $(,)?) => {
        unsafe fn $name($($arg: usize),*) {
            unsafe {
                core::arch::asm!(
                    $($line,)+
                    $($arg = in(reg) $arg,)*
                    out("eax") _,
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            }
        }
    };
}
pub(crate) use reference_eax;

/// What the block reads at a place: a float, a whole number, an 80-bit value.
#[derive(Clone, Copy)]
pub enum In {
    F,
    I,
    T,
}

/// What the block writes at a place: a float, a whole number, an 80-bit value, the x87's
/// status word (its condition bits C3, C2, C1, C0 are compared).
#[derive(Clone, Copy)]
pub enum Out {
    F,
    I,
    T,
    H,
}

/// An operand of a block: the memory the register addresses (the places read and written, as
/// offsets from it) or a value (an index).
pub enum Operand {
    Buffer {
        reads: &'static [(usize, In)],
        writes: &'static [(usize, Out)],
    },
    Value(usize),
    /// The race block itself (`race::store::asm_base()`): the places are RVAs, read and
    /// written where the typed stores see them; put back after the block's cases.
    RaceBlock {
        reads: &'static [(usize, In)],
        writes: &'static [(usize, Out)],
    },
    /// The image's zeroed tail (`crate::race::owner::race().shared.buffers`), as `RaceBlock`: the places
    /// are RVAs; the register is the static's address less its first byte's RVA.
    Buffers {
        reads: &'static [(usize, In)],
        writes: &'static [(usize, Out)],
    },
}

/// An operand's places read and written.
type Places = (&'static [(usize, In)], &'static [(usize, Out)]);

impl Operand {
    fn places(&self) -> Option<Places> {
        match *self {
            Operand::Buffer { reads, writes }
            | Operand::RaceBlock { reads, writes }
            | Operand::Buffers { reads, writes } => Some((reads, writes)),
            Operand::Value(_) => None,
        }
    }
}

/// An operand's memory: a buffer of the test's, or the race block at its RVAs.
enum Mem {
    Buffer(Vec<u8>),
    Block,
    None,
}

impl Mem {
    fn get(&self, off: usize, len: usize) -> Vec<u8> {
        match self {
            Mem::Buffer(v) => v[off..off + len].to_vec(),
            Mem::Block => crate::pointers::GamePtr::new(block_at(off))
                .read_bytes(len, "a race block place")
                .unwrap(),
            Mem::None => Vec::new(),
        }
    }

    fn put(&mut self, off: usize, bytes: &[u8]) {
        match self {
            Mem::Buffer(v) => v[off..off + bytes.len()].copy_from_slice(bytes),
            Mem::Block => crate::pointers::GamePtr::new(block_at(off))
                .write_bytes(bytes, "a race block place")
                .unwrap(),
            Mem::None => {}
        }
    }
}

fn block_at(off: usize) -> u32 {
    crate::framework::game_addr(off)
}

pub struct Block {
    pub name: &'static str,
    pub operands: Vec<Operand>,
    /// How many float ulps apart a result may be (a transcendental's: libm's, not the x87's).
    pub ulps: i64,
    pub reference: fn(&[usize]),
    pub plain: fn(&[usize]),
}

fn width_in(k: In) -> usize {
    match k {
        In::T => 10,
        _ => 4,
    }
}

fn width_out(k: Out) -> usize {
    match k {
        Out::T => 10,
        Out::H => 2,
        _ => 4,
    }
}

/// The race block's places put back (also when a comparison fails).
struct PutBack(Vec<(usize, Vec<u8>)>);

impl Drop for PutBack {
    fn drop(&mut self) {
        for (off, bytes) in self.0.iter().rev() {
            Mem::Block.put(*off, bytes);
        }
    }
}

/// Runs each block `cases` times through the reference and the plain version at every control
/// word, on seeded random inputs at the places the block reads; compares every place it
/// writes (floats equal or both NaN, whole numbers and 80-bit values bit for bit, the status
/// words' condition bits).
pub fn sparse(blocks: &[Block], cases: u32, seed: u64) {
    sparse_at(blocks, cases, seed, &CONTROL_WORDS);
}

/// `sparse` at the control words `words` only: for plain code written for the rounding the
/// game runs it at (to nearest), which the reference assembly follows in every mode.
pub fn sparse_at(blocks: &[Block], cases: u32, seed: u64, words: &[u16]) {
    let _g = lock();
    let _restore = Restore(control_word());
    let mut rng = Rng::new(seed);
    const SIMPLE: [f32; 5] = [0.0, -0.0, 1.0, -1.0, 0.5];
    for &cw in words {
        set_control_word(cw);
        for block in blocks {
            let mut buffers: Vec<Mem> = Vec::new();
            let mut args: Vec<usize> = Vec::new();
            // the race block's places as they were, put back after the cases
            let mut saved: Vec<(usize, Vec<u8>)> = Vec::new();
            for o in &block.operands {
                match o {
                    Operand::Buffer { reads, writes } => {
                        let end = reads
                            .iter()
                            .map(|&(off, k)| off + width_in(k))
                            .chain(writes.iter().map(|&(off, k)| off + width_out(k)))
                            .max()
                            .unwrap_or(0);
                        let mut v = vec![0u8; end + 16];
                        args.push((crate::pointers::game_address(v.as_mut_ptr())) as usize);
                        buffers.push(Mem::Buffer(v));
                    }
                    Operand::RaceBlock { reads, writes } | Operand::Buffers { reads, writes } => {
                        args.push(match o {
                            Operand::Buffers { .. } => crate::race::buffers_store::home()
                                .wrapping_sub(crate::framework::rva(
                                    crate::race::buffers_store::START,
                                )),
                            _ => crate::race::store::asm_base() as usize,
                        });
                        let places = reads
                            .iter()
                            .map(|&(off, k)| (off, width_in(k)))
                            .chain(writes.iter().map(|&(off, k)| (off, width_out(k))));
                        saved.extend(places.map(|(off, len)| (off, Mem::Block.get(off, len))));
                        buffers.push(Mem::Block);
                    }
                    Operand::Value(x) => {
                        args.push(*x);
                        buffers.push(Mem::None);
                    }
                }
            }
            // The buffers lent while the block runs: its operands are checked as game memory.
            let _lent: Vec<crate::pointers::Lent> = buffers
                .iter()
                .filter_map(|b| match b {
                    Mem::Buffer(v) => Some(crate::pointers::Lent::new(v)),
                    _ => None,
                })
                .collect();
            let _put_back = PutBack(saved);
            // The race block's places the reference reads or writes (IDA's addresses), laid
            // into their old places around it (`race::store::OldPlaces`).
            let old_places: Vec<usize> = block
                .operands
                .iter()
                .filter_map(|o| match o {
                    Operand::RaceBlock { reads, writes } => Some(
                        reads
                            .iter()
                            .map(|&(off, _)| off)
                            .chain(writes.iter().map(|&(off, _)| off))
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                })
                .flatten()
                .map(|off| off + 0x400000)
                .collect();
            // the places written, as (operand, offset, kind)
            let mut places: Vec<(usize, usize, Out)> = Vec::new();
            for (n, o) in block.operands.iter().enumerate() {
                if let Some((_, writes)) = o.places() {
                    places.extend(writes.iter().map(|&(off, k)| (n, off, k)));
                }
            }
            let (mut nans, mut floats) = (0u32, 0u32);
            for case in 0..cases {
                let moderate = rng.below(2) == 0;
                let simple = rng.below(8) == 0;
                for (o, buf) in block.operands.iter().zip(buffers.iter_mut()) {
                    let Some((reads, writes)) = o.places() else {
                        continue;
                    };
                    for &(off, k) in writes.iter() {
                        buf.put(off, &[0xAA; 10][..width_out(k)]);
                    }
                    for &(off, k) in reads.iter() {
                        match k {
                            In::F => {
                                let bits = if simple {
                                    SIMPLE[rng.below(5) as usize].to_bits()
                                } else if moderate {
                                    rng.moderate()
                                } else {
                                    rng.fl()
                                };
                                buf.put(off, &bits.to_le_bytes());
                            }
                            In::I => {
                                let v = if simple {
                                    rng.below(3) as u32
                                } else {
                                    rng.int()
                                };
                                buf.put(off, &v.to_le_bytes());
                            }
                            In::T => {
                                let bits = if moderate { rng.moderate() } else { rng.fl() };
                                let mut m = [0u32; 3];
                                put_t80(&mut m, 0, bits);
                                let bytes: [u8; 12] = unsafe { core::mem::transmute(m) };
                                buf.put(off, &bytes[..10]);
                            }
                        }
                    }
                }
                // the inputs: every place the block reads or writes, kept to run it twice
                let mut touched: Vec<(usize, usize, Vec<u8>)> = Vec::new();
                for (n, o) in block.operands.iter().enumerate() {
                    if let Some((reads, writes)) = o.places() {
                        for &(off, k) in reads.iter() {
                            touched.push((n, off, buffers[n].get(off, width_in(k))));
                        }
                        for &(off, k) in writes.iter() {
                            touched.push((n, off, buffers[n].get(off, width_out(k))));
                        }
                    }
                }
                let read_out = |buffers: &Vec<Mem>| -> Vec<Vec<u8>> {
                    places
                        .iter()
                        .map(|&(n, off, k)| buffers[n].get(off, width_out(k)))
                        .collect()
                };
                crate::race::store::in_old_places(&old_places, || (block.reference)(&args));
                let from_reference = read_out(&buffers);
                for (n, off, bytes) in &touched {
                    buffers[*n].put(*off, bytes);
                }
                (block.plain)(&args);
                let from_plain = read_out(&buffers);
                for (k, &(n, off, kind)) in places.iter().enumerate() {
                    let (a, b) = (&from_reference[k], &from_plain[k]);
                    let ok = match kind {
                        Out::F => {
                            let (x, y) = (
                                u32::from_le_bytes(a[..4].try_into().unwrap()),
                                u32::from_le_bytes(b[..4].try_into().unwrap()),
                            );
                            floats += 1;
                            nans += u32::from(f32::from_bits(x).is_nan());
                            close_float(x, y, block.ulps)
                        }
                        Out::I | Out::T => a == b,
                        Out::H => {
                            let (x, y) = (
                                u16::from_le_bytes(a[..2].try_into().unwrap()),
                                u16::from_le_bytes(b[..2].try_into().unwrap()),
                            );
                            x & 0x4700 == y & 0x4700
                        }
                    };
                    assert!(
                        ok,
                        "{}: control word {cw:#06X}, case {case}, operand {n} offset {off:#X}: reference {a:02X?}, plain {b:02X?}",
                        block.name
                    );
                }
            }
            // results that are all NaN compare equal whatever they are
            assert!(
                nans * 4 <= floats * 3,
                "{}: {nans} of {floats} compared floats are NaN",
                block.name
            );
        }
    }
}
