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

/// Nearest (the game's), chop (the bug-hunting one), down, up.
pub const CONTROL_WORDS: [u16; 4] = [0x027F, 0x0E7F, 0x067F, 0x0A7F];

pub struct Rng(u64);

const EDGES: [f32; 24] = [
    0.0,
    -0.0,
    1.0,
    -1.0,
    0.5,
    -0.5,
    1.5,
    2.5,
    -2.5,
    0.25,
    1e-45,
    1.1754942e-38,
    -1e-45,
    1e30,
    1e-30,
    2147483520.0,
    2147483648.0,
    -2147483648.0,
    16777216.0,
    16777217.0,
    std::f32::consts::PI,
    std::f32::consts::FRAC_PI_2,
    4096.0,
    65536.0,
];

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    /// A uniform f32 in [-1, 1).
    pub fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 23) as f32 - 1.0
    }

    /// A float's bits: edge values, the unit range, game-sized values, integers, halves,
    /// large and tiny values, any finite pattern. Never a NaN or an infinity.
    pub fn fl(&mut self) -> u32 {
        match self.below(12) {
            0 => EDGES[self.below(EDGES.len() as u64) as usize].to_bits(),
            1 | 2 => self.unit().to_bits(),
            3 | 4 => (self.unit() * 1000.0).to_bits(),
            5 => (self.small() as f32).to_bits(),
            6 => (self.small() as f32 * 0.5).to_bits(),
            7 => (self.unit() * 1e6).to_bits(),
            8 => (self.unit() * 1e-3).to_bits(),
            9 => {
                let b = self.next() as u32;
                if (b >> 23) & 0xFF == 0xFF {
                    b & !(1 << 30)
                } else {
                    b
                }
            }
            10 => (self.unit() * 100.0).to_bits(),
            _ => (self.unit() * 1e9).to_bits(),
        }
    }

    /// A float that is not a huge or tiny value (for chains that would overflow).
    pub fn moderate(&mut self) -> u32 {
        match self.below(4) {
            0 => (self.unit() * 100.0).to_bits(),
            1 => self.unit().to_bits(),
            2 => (self.unit() * 5000.0).to_bits(),
            _ => (self.small() as f32 * 0.25).to_bits(),
        }
    }

    /// An angle in radians (bits): the turns of a game's objects, zero and the exact multiples
    /// of the float pi among them. Never large (the x87 and libm reduce large ones differently).
    pub fn angle(&mut self) -> u32 {
        match self.below(8) {
            0 => 0.0f32.to_bits(),
            1 => (std::f32::consts::FRAC_PI_2 * (self.below(9) as f32 - 4.0)).to_bits(),
            2 | 3 => (self.unit() * 7.0).to_bits(),
            4 => (self.unit() * 100.0).to_bits(),
            5 => (self.unit() * 0.01).to_bits(),
            6 => (self.small() as f32 * 0.01).to_bits(),
            _ => (self.unit() * 1000.0).to_bits(),
        }
    }

    /// A small whole number, positive or negative.
    pub fn small(&mut self) -> i32 {
        (self.below(20001) as i32) - 10000
    }

    /// A whole number: zero, small, game-sized, near the edges of the type.
    pub fn int(&mut self) -> u32 {
        (match self.below(8) {
            0 => 0,
            1 | 2 => self.small(),
            3 => (self.unit() * 1.0e6) as i32,
            4 => (self.unit() * 2.0e7) as i32,
            5 => self.next() as i32,
            6 => i32::MAX - self.below(300) as i32,
            _ => i32::MIN + self.below(300) as i32,
        }) as u32
    }
}

/// The same float, or both a NaN.
pub fn same_float(a: u32, b: u32) -> bool {
    a == b || (f32::from_bits(a).is_nan() && f32::from_bits(b).is_nan())
}

/// Within `ulps` units in the last place (a NaN against a NaN, an exact zero against a zero).
pub fn close_float(a: u32, b: u32, ulps: i64) -> bool {
    if same_float(a, b) {
        return true;
    }
    let (x, y) = (f32::from_bits(a), f32::from_bits(b));
    if x.is_nan() || y.is_nan() {
        return false;
    }
    let key = |v: u32| -> i64 {
        if v & 0x8000_0000 != 0 {
            -((v & 0x7FFF_FFFF) as i64)
        } else {
            v as i64
        }
    };
    (key(a) - key(b)).abs() <= ulps
}

struct Restore(u16);
impl Drop for Restore {
    fn drop(&mut self) {
        set_control_word(self.0);
    }
}

/// How one dword of the block's memory is compared.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Bit for bit.
    Exact,
    /// A float: bit for bit, or both a NaN.
    Float,
    /// A float of a transcendental: one ulp apart is allowed.
    Ulp1,
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

/// Writes the float with these bits, as an 80-bit value, `at` bytes into the image.
pub fn put_t80<const N: usize>(m: &mut [u32; N], at: usize, bits: u32) {
    use crate::math::x87_float::ld;
    crate::scene::x87_mem::fstp80(m.as_mut_ptr().cast(), at, ld(bits))
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

/// Runs each block `cases` times through the reference and the plain version at every control
/// word, on seeded random inputs at the places the block reads; compares every place it
/// writes (floats equal or both NaN, whole numbers and 80-bit values bit for bit, the status
/// words' condition bits).
pub fn sparse(blocks: &[Block], cases: u32, seed: u64) {
    let _g = lock();
    let _restore = Restore(control_word());
    let mut rng = Rng::new(seed);
    const SIMPLE: [f32; 5] = [0.0, -0.0, 1.0, -1.0, 0.5];
    for &cw in &CONTROL_WORDS {
        set_control_word(cw);
        for block in blocks {
            let mut buffers: Vec<Vec<u8>> = Vec::new();
            let mut args: Vec<usize> = Vec::new();
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
                        buffers.push(v);
                    }
                    Operand::Value(x) => {
                        args.push(*x);
                        buffers.push(Vec::new());
                    }
                }
            }
            // The buffers lent while the block runs: its operands are checked as game memory.
            let _lent: Vec<crate::pointers::Lent> = buffers
                .iter()
                .filter(|b| !b.is_empty())
                .map(|b| crate::pointers::Lent::new(b))
                .collect();
            // the places written, as (operand, offset, kind)
            let mut places: Vec<(usize, usize, Out)> = Vec::new();
            for (n, o) in block.operands.iter().enumerate() {
                if let Operand::Buffer { writes, .. } = o {
                    places.extend(writes.iter().map(|&(off, k)| (n, off, k)));
                }
            }
            let (mut nans, mut floats) = (0u32, 0u32);
            for case in 0..cases {
                let moderate = rng.below(2) == 0;
                let simple = rng.below(8) == 0;
                for (o, buf) in block.operands.iter().zip(buffers.iter_mut()) {
                    let Operand::Buffer { reads, writes } = o else {
                        continue;
                    };
                    for &(off, k) in writes.iter() {
                        buf[off..off + width_out(k)].fill(0xAA);
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
                                buf[off..off + 4].copy_from_slice(&bits.to_le_bytes());
                            }
                            In::I => {
                                let v = if simple {
                                    rng.below(3) as u32
                                } else {
                                    rng.int()
                                };
                                buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
                            }
                            In::T => {
                                let bits = if moderate { rng.moderate() } else { rng.fl() };
                                let mut m = [0u32; 3];
                                put_t80(&mut m, 0, bits);
                                let bytes: [u8; 12] = unsafe { core::mem::transmute(m) };
                                buf[off..off + 10].copy_from_slice(&bytes[..10]);
                            }
                        }
                    }
                }
                // the inputs: every place the block reads or writes, kept to run it twice
                let mut touched: Vec<(usize, usize, Vec<u8>)> = Vec::new();
                for (n, o) in block.operands.iter().enumerate() {
                    if let Operand::Buffer { reads, writes } = o {
                        for &(off, k) in reads.iter() {
                            touched.push((n, off, buffers[n][off..off + width_in(k)].to_vec()));
                        }
                        for &(off, k) in writes.iter() {
                            touched.push((n, off, buffers[n][off..off + width_out(k)].to_vec()));
                        }
                    }
                }
                let read_out = |buffers: &Vec<Vec<u8>>| -> Vec<Vec<u8>> {
                    places
                        .iter()
                        .map(|&(n, off, k)| buffers[n][off..off + width_out(k)].to_vec())
                        .collect()
                };
                (block.reference)(&args);
                let from_reference = read_out(&buffers);
                for (n, off, bytes) in &touched {
                    buffers[*n][*off..*off + bytes.len()].copy_from_slice(bytes);
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
