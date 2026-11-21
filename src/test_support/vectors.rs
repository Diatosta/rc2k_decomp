//! The reference vectors: what the 1999 x87 blocks made of the tests' inputs, recorded once on i686
//! (where the blocks' instructions could run) and replayed against the Rust codes on every target.
//!
//! The inputs are not stored: each test makes them again from its seed with `Rng`, and the
//! file holds a hash of the whole input stream, so a test whose inputs changed is told so
//! instead of comparing against another test's results. What is stored is each case's
//! results, as bytes exclusive-or'd with the same bytes before the block ran (the places a
//! block leaves alone are zeros), the whole deflated.
//!
//! A file, `data/vectors/<name>.bin`, after inflating: the magic `RC2KVEC1`, the input
//! stream's FNV-1a (u64), the number of result bytes (u32), the result bytes; all little
//! endian.
//!
//! `differential` replays a test over blocks of dwords, `sparse` over a few places of operand
//! buffers; `Recorder` is the general form. The instructions the files were recorded from are gone
//! from the tests: the files are replayed only. (`Recorder::record` still writes a file, with
//! `RC2K_RECORD_VECTORS=1`, from whatever results its caller gives.)

// reason: each target uses part of the helpers.
#![allow(dead_code)]

use crate::math::x87::{control_word, set_control_word};
use std::cell::Cell;

/// Nearest (the game's), chop (the bug-hunting one), down, up.
pub const CONTROL_WORDS: [u16; 4] = [0x027F, 0x0E7F, 0x067F, 0x0A7F];

/// The tests' seeded random numbers (splitmix64) and the shapes of float they draw.
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
fn same_float(a: u32, b: u32) -> bool {
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

/// Writes the float with these bits, as an 80-bit value, `at` bytes into the block.
pub fn put_t80<const N: usize>(m: &mut [u32; N], at: usize, bits: u32) {
    use crate::math::x87_float::{f80, ld};
    let value = f80(ld(bits));
    let mut bytes: Vec<u8> = m.iter().flat_map(|d| d.to_le_bytes()).collect();
    bytes[at..at + 10].copy_from_slice(&value);
    for (d, b) in m.iter_mut().zip(bytes.as_chunks::<4>().0) {
        *d = u32::from_le_bytes(*b);
    }
}

/// A test's block of dwords as cells.
pub fn cells<const N: usize>(m: &mut [u32; N]) -> &[Cell<u32>] {
    Cell::from_mut(&mut m[..]).as_slice_of_cells()
}

/// A test's block of dwords as the bytes it starts at (for a Rust version over a byte
/// pointer: the cells may be written through it).
pub fn block_bytes(c: &[Cell<u32>]) -> *mut u8 {
    c.as_ptr().cast_mut().cast::<u8>()
}

// ---- the file ----

const MAGIC: &[u8; 8] = b"RC2KVEC1";

/// The folder of the recorded vectors: `data/vectors` of the source tree, or `RC2K_VECTORS`
/// for a test binary run away from it (the Windows suite on another machine).
pub(super) fn vectors_dir() -> std::path::PathBuf {
    std::env::var_os("RC2K_VECTORS").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/vectors"),
        std::path::PathBuf::from,
    )
}

/// Where the vectors named `name` are kept.
pub fn path(name: &str) -> std::path::PathBuf {
    vectors_dir().join(format!("{name}.bin"))
}

/// Whether this run records (`RC2K_RECORD_VECTORS=1`).
pub fn recording() -> bool {
    std::env::var("RC2K_RECORD_VECTORS").is_ok_and(|v| v == "1")
}

/// The FNV-1a hash of the input stream as it is fed in.
struct Fnv(u64);

impl Fnv {
    fn feed(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(0x0100_0000_01B3);
        }
    }
}

/// A test's run through its vectors: its inputs hashed as they are made, each case's
/// results taken from the file (replaying) or from the reference (recording).
pub struct Recorder {
    name: String,
    inputs: Fnv,
    /// Recording: the results so far; replaying: the file's.
    results: Vec<u8>,
    /// Replaying: the file's input hash and the next result byte.
    replay: Option<(u64, usize)>,
}

impl Recorder {
    /// The vectors named `name` to replay (the test fails if they were never recorded).
    pub fn replay(name: &str) -> Recorder {
        let path = path(name);
        let packed = std::fs::read(&path).unwrap_or_else(|e| {
            panic!(
                "{name}: no vectors at {} ({e}): record them on i686 with RC2K_RECORD_VECTORS=1",
                path.display()
            )
        });
        let data = miniz_oxide::inflate::decompress_to_vec_zlib(&packed)
            .unwrap_or_else(|e| panic!("{name}: the vectors do not inflate: {e:?}"));
        assert!(
            data.len() >= 20 && &data[..8] == MAGIC,
            "{name}: not a vector file"
        );
        let hash = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let len = u32::from_le_bytes(data[16..20].try_into().unwrap()) as usize;
        assert_eq!(data.len(), 20 + len, "{name}: the vector file's length");
        Recorder {
            name: name.to_string(),
            inputs: Fnv(0xCBF2_9CE4_8422_2325),
            results: data[20..].to_vec(),
            replay: Some((hash, 0)),
        }
    }

    /// A new recording named `name`.
    pub fn record(name: &str) -> Recorder {
        Recorder {
            name: name.to_string(),
            inputs: Fnv(0xCBF2_9CE4_8422_2325),
            results: Vec::new(),
            replay: None,
        }
    }

    /// The case's input, part of the stream the file was recorded for.
    pub fn input(&mut self, bytes: &[u8]) {
        self.inputs.feed(bytes);
    }

    /// The reference's results of a case, as long as `before` (the same places before the
    /// block ran): recorded from `reference`, or read from the file.
    pub fn results(&mut self, before: &[u8], reference: impl FnOnce() -> Vec<u8>) -> Vec<u8> {
        match &mut self.replay {
            Some((_, at)) => {
                let end = *at + before.len();
                assert!(
                    end <= self.results.len(),
                    "{}: the test asks for more results than were recorded",
                    self.name
                );
                let out = self.results[*at..end]
                    .iter()
                    .zip(before)
                    .map(|(r, b)| r ^ b)
                    .collect();
                *at = end;
                out
            }
            None => {
                let made = reference();
                assert_eq!(made.len(), before.len(), "{}: a result's length", self.name);
                self.results
                    .extend(made.iter().zip(before).map(|(r, b)| r ^ b));
                made
            }
        }
    }

    /// A case's result of a length the test does not know beforehand (a returned value's
    /// printed form): recorded from `reference`, or read from the file.
    pub fn result_bytes(&mut self, reference: impl FnOnce() -> Vec<u8>) -> Vec<u8> {
        let mut made = None;
        let len = self.results(&[0; 4], || {
            let bytes = reference();
            let len = (bytes.len() as u32).to_le_bytes().to_vec();
            made = Some(bytes);
            len
        });
        let len = u32::from_le_bytes(len.try_into().unwrap()) as usize;
        self.results(&vec![0; len], || made.expect("a recorded result"))
    }

    /// The replay checked whole (the same inputs, every result used), or the recording
    /// written.
    pub fn finish(self) {
        let hash = self.inputs.0;
        match self.replay {
            Some((recorded, at)) => {
                assert_eq!(
                    recorded, hash,
                    "{}: the test's inputs are not the ones its vectors were recorded for",
                    self.name
                );
                assert_eq!(at, self.results.len(), "{}: results left unused", self.name);
            }
            None => {
                let mut data = MAGIC.to_vec();
                data.extend(hash.to_le_bytes());
                data.extend((self.results.len() as u32).to_le_bytes());
                data.extend(&self.results);
                let packed = miniz_oxide::deflate::compress_to_vec_zlib(&data, 10);
                let path = path(&self.name);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(&path, packed)
                    .unwrap_or_else(|e| panic!("{}: writing {}: {e}", self.name, path.display()));
            }
        }
    }
}

struct Restore(u16);
impl Drop for Restore {
    fn drop(&mut self) {
        set_control_word(self.0);
    }
}

// ---- blocks of dwords ----

/// How one dword of the block is compared.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Bit for bit.
    Exact,
    /// A float: bit for bit, or both a NaN.
    Float,
    /// A float of a transcendental: one ulp apart is allowed.
    Ulp1,
}

/// One test over blocks of `N` dwords: `cases` seeded random blocks at each control word, `fill`
/// writing one case's input, `plain` the Rust code over the block.
pub struct Differential<'a, const N: usize> {
    pub name: &'a str,
    pub cases: u32,
    pub seed: u64,
    pub kinds: [Kind; N],
    pub fill: &'a dyn Fn(&mut Rng, &mut [u32; N]),
    pub plain: &'a dyn Fn(&[Cell<u32>]),
}

/// Replays the test's vectors: each block through the Rust code, compared dword by dword with the
/// reference's results as `kinds` say.
pub fn differential<const N: usize>(test: Differential<N>) {
    run_differential(test, Recorder::replay(test.name));
}

impl<const N: usize> Clone for Differential<'_, N> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<const N: usize> Copy for Differential<'_, N> {}

fn bytes_of<const N: usize>(m: &[u32; N]) -> Vec<u8> {
    m.iter().flat_map(|d| d.to_le_bytes()).collect()
}

// The lock: a replay holds no test lock (`test_support::locks`): the locks are for the tests that
// write the process-wide statics, and a replay reaches none (the race block's places it names are
// the stores of the test thread's own owners). The control word it sets
// (`set_control_word`) is this thread's: the emulated one is a `thread_local` (`math::x87`). The
// inputs, the Rust code's operands and the results are the test's own buffers; the file is read
// only. And every caller's Rust version (`plain`) works on its operands only: checked for every
// caller of `differential` (`input::update_car_effects`, `scene::{add_car_light, clip_asm,
// cross_product3, face_faces_away, light_dot, matrix3, place_object_points, project_point,
// reflection_coordinates, reflect_vector, rotate_point, vector3}`) and of `sparse`
// (`hud::{disc_vertex, draw_disc, draw_needle}`): the blocks are x87 arithmetic over the cells
// given, records laid over the test's buffers (`Arg::view`), no owner, store or other static. A new
// caller whose Rust version reaches a process-wide static takes the lock of that static
// (`test_support::locks`) around its test.

fn run_differential<const N: usize>(test: Differential<N>, mut file: Recorder) {
    // No lock ("The lock" above).
    let _restore = Restore(control_word());
    let name = test.name;
    let mut rng = Rng::new(test.seed);
    let (mut compared, mut nans) = (0u64, 0u64);
    for &cw in &CONTROL_WORDS {
        set_control_word(cw);
        for case in 0..test.cases {
            let mut input = [0u32; N];
            (test.fill)(&mut rng, &mut input);
            let before = bytes_of(&input);
            file.input(&before);
            let made = file.results(&before, || unreachable!("{name}: replayed only"));
            let a: [u32; N] = std::array::from_fn(|k| {
                u32::from_le_bytes(made[4 * k..4 * k + 4].try_into().unwrap())
            });
            let mut b = input;
            (test.plain)(cells(&mut b));
            for k in 0..N {
                if matches!(test.kinds[k], Kind::Float | Kind::Ulp1) {
                    compared += 1;
                    nans += u64::from(f32::from_bits(a[k]).is_nan());
                }
                let ok = match test.kinds[k] {
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
    file.finish();
}

// ---- a few places of operand buffers ----

/// What a block reads at a place: a float, a whole number, an 80-bit value.
#[derive(Clone, Copy)]
pub enum In {
    F,
    I,
    T,
}

/// What a block writes at a place: a float, a whole number, an 80-bit value, the x87's
/// status word (its condition bits C3, C2, C1, C0 are compared).
#[derive(Clone, Copy)]
pub enum Out {
    F,
    I,
    T,
    H,
}

/// An operand of a block: a buffer (the places read and written, as byte offsets into it)
/// or a value (an index).
pub enum Operand {
    Buffer {
        reads: &'static [(usize, In)],
        writes: &'static [(usize, Out)],
    },
    Value(usize),
    /// A buffer at least `len` bytes long (one a typed store is laid over, `Arg::view`).
    Sized {
        len: usize,
        reads: &'static [(usize, In)],
        writes: &'static [(usize, Out)],
    },
}

/// An operand as the Rust code gets it.
pub enum Arg<'a> {
    Buffer(&'a [Cell<u32>]),
    Value(usize),
}

impl<'a> Arg<'a> {
    /// A buffer operand's dwords.
    pub fn cells(&self) -> &'a [Cell<u32>] {
        match *self {
            Arg::Buffer(c) => c,
            Arg::Value(_) => panic!("a value operand used as a buffer"),
        }
    }

    /// A value operand.
    pub fn value(&self) -> usize {
        match *self {
            Arg::Value(v) => v,
            Arg::Buffer(_) => panic!("a buffer operand used as a value"),
        }
    }

    /// A buffer operand seen as the `T` laid over its first bytes (a store made of cells).
    pub fn view<T>(&self) -> &'a T {
        let c = self.cells();
        assert!(
            size_of_val(c) >= size_of::<T>(),
            "a buffer too small for its view"
        );
        assert!(
            align_of::<T>() <= align_of::<Cell<u32>>(),
            "a view's alignment"
        );
        // SAFETY: the buffer covers the `T` and is aligned for it; the `T`s viewed are stores
        // of cells, valid at any bits.
        unsafe { &*c.as_ptr().cast::<T>() }
    }
}

/// A block of a sparse test: its operands, how many float ulps apart a result may be (a
/// transcendental's: libm's, not the x87's) and the Rust code.
pub struct Block {
    pub name: &'static str,
    pub operands: Vec<Operand>,
    pub ulps: i64,
    pub plain: fn(&[Arg]),
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

/// An operand's places read and written.
type Places = (&'static [(usize, In)], &'static [(usize, Out)]);

fn places(o: &Operand) -> Option<Places> {
    match *o {
        Operand::Buffer { reads, writes } | Operand::Sized { reads, writes, .. } => {
            Some((reads, writes))
        }
        Operand::Value(_) => None,
    }
}

fn get(buf: &[u32], off: usize, len: usize) -> Vec<u8> {
    let bytes: &[u8] = bytemuck_bytes(buf);
    bytes[off..off + len].to_vec()
}

fn put(buf: &mut [u32], off: usize, value: &[u8]) {
    let mut bytes = bytemuck_bytes(buf).to_vec();
    bytes[off..off + value.len()].copy_from_slice(value);
    for (d, b) in buf.iter_mut().zip(bytes.as_chunks::<4>().0) {
        *d = u32::from_le_bytes(*b);
    }
}

fn bytemuck_bytes(buf: &[u32]) -> &[u8] {
    // SAFETY: a `u32` slice's bytes, any bits valid as `u8`.
    unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), size_of_val(buf)) }
}

/// Replays the sparse test `name`: each block `cases` times at every control word, on
/// seeded random inputs at the places it reads; every place it writes compared with the
/// reference's results (floats within the block's ulps or both NaN, whole numbers and 80-bit
/// values bit for bit, the status words' condition bits).
pub fn sparse(name: &str, blocks: &[Block], cases: u32, seed: u64) {
    sparse_at(name, blocks, cases, seed, &CONTROL_WORDS);
}

/// `sparse` at the control words `words` only.
pub fn sparse_at(name: &str, blocks: &[Block], cases: u32, seed: u64, words: &[u16]) {
    run_sparse(Recorder::replay(name), blocks, cases, seed, words);
}

fn run_sparse(mut file: Recorder, blocks: &[Block], cases: u32, seed: u64, words: &[u16]) {
    // As `run_differential`: no lock ("The lock" above).
    let _restore = Restore(control_word());
    let mut rng = Rng::new(seed);
    const SIMPLE: [f32; 5] = [0.0, -0.0, 1.0, -1.0, 0.5];
    for &cw in words {
        set_control_word(cw);
        for block in blocks.iter() {
            let mut buffers: Vec<Vec<u32>> = block
                .operands
                .iter()
                .map(|o| match places(o) {
                    Some((reads, writes)) => {
                        let end = reads
                            .iter()
                            .map(|&(off, k)| off + width_in(k))
                            .chain(writes.iter().map(|&(off, k)| off + width_out(k)))
                            .max()
                            .unwrap_or(0);
                        let len = match o {
                            Operand::Sized { len, .. } => (end + 16).max(*len),
                            _ => end + 16,
                        };
                        vec![0u32; len.div_ceil(4)]
                    }
                    None => Vec::new(),
                })
                .collect();
            // the places written, as (operand, offset, kind)
            let mut written: Vec<(usize, usize, Out)> = Vec::new();
            for (n, o) in block.operands.iter().enumerate() {
                if let Some((_, writes)) = places(o) {
                    written.extend(writes.iter().map(|&(off, k)| (n, off, k)));
                }
            }
            let read_out = |buffers: &Vec<Vec<u32>>| -> Vec<u8> {
                written
                    .iter()
                    .flat_map(|&(n, off, k)| get(&buffers[n], off, width_out(k)))
                    .collect()
            };
            let (mut nans, mut floats) = (0u32, 0u32);
            for case in 0..cases {
                let moderate = rng.below(2) == 0;
                let simple = rng.below(8) == 0;
                for (o, buf) in block.operands.iter().zip(buffers.iter_mut()) {
                    let Some((reads, writes)) = places(o) else {
                        continue;
                    };
                    for &(off, k) in writes.iter() {
                        put(buf, off, &[0xAA; 10][..width_out(k)]);
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
                                put(buf, off, &bits.to_le_bytes());
                            }
                            In::I => {
                                let v = if simple {
                                    rng.below(3) as u32
                                } else {
                                    rng.int()
                                };
                                put(buf, off, &v.to_le_bytes());
                            }
                            In::T => {
                                let bits = if moderate { rng.moderate() } else { rng.fl() };
                                let mut m = [0u32; 3];
                                put_t80(&mut m, 0, bits);
                                put(buf, off, &bytemuck_bytes(&m)[..10]);
                            }
                        }
                    }
                }
                let start = buffers.clone();
                for (o, buf) in block.operands.iter().zip(&buffers) {
                    if let Some((reads, writes)) = places(o) {
                        for &(off, k) in reads.iter() {
                            file.input(&get(buf, off, width_in(k)));
                        }
                        for &(off, k) in writes.iter() {
                            file.input(&get(buf, off, width_out(k)));
                        }
                    }
                }
                let before = read_out(&buffers);
                let expected = file.results(&before, || unreachable!("replayed only"));
                let mut run = start;
                let made = {
                    let args: Vec<Arg> = block
                        .operands
                        .iter()
                        .zip(run.iter_mut())
                        .map(|(o, b)| match o {
                            Operand::Value(v) => Arg::Value(*v),
                            _ => Arg::Buffer(Cell::from_mut(&mut b[..]).as_slice_of_cells()),
                        })
                        .collect();
                    (block.plain)(&args);
                    read_out(&run)
                };
                let mut at = 0;
                for &(n, off, kind) in &written {
                    let len = width_out(kind);
                    let (a, b) = (&expected[at..at + len], &made[at..at + len]);
                    at += len;
                    let ok = match kind {
                        Out::F => {
                            let (x, y) = (
                                u32::from_le_bytes(a.try_into().unwrap()),
                                u32::from_le_bytes(b.try_into().unwrap()),
                            );
                            floats += 1;
                            nans += u32::from(f32::from_bits(x).is_nan());
                            close_float(x, y, block.ulps)
                        }
                        Out::I | Out::T => a == b,
                        Out::H => {
                            let (x, y) = (
                                u16::from_le_bytes(a.try_into().unwrap()),
                                u16::from_le_bytes(b.try_into().unwrap()),
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
    file.finish();
}

// ---- blocks over the race block's places (lane t5rest) ----

/// Blocks that read and write places of the race block (by their offsets from its start) and of
/// buffers of the test's own, the Rust code run on the typed stores: the 1999 results recorded
/// once, replayed against `plain`.
pub mod block_vectors {
    use super::{CONTROL_WORDS, Recorder, Restore, Rng, close_float, put_t80};
    use crate::math::x87::{control_word, set_control_word};

    pub use super::{In, Out};

    /// An operand of a block: the memory a register addresses (the places read and written,
    /// as offsets from it) or a value (an index).
    pub enum Operand {
        Buffer {
            reads: &'static [(usize, In)],
            writes: &'static [(usize, Out)],
        },
        /// A buffer of at least `len` bytes, for a record the Rust code holds in its own layout
        /// (`sound::asm_layout`): it reads the whole 1999 extent from the buffer.
        Sized {
            len: usize,
            reads: &'static [(usize, In)],
            writes: &'static [(usize, Out)],
        },
        Value(usize),
        /// The race block itself: the places are offsets from its start, read and written
        /// where the typed stores see them; put back after the block's cases.
        RaceBlock {
            reads: &'static [(usize, In)],
            writes: &'static [(usize, Out)],
        },
        /// A buffer of the test's own standing for the race's buffers, as `RaceBlock`.
        Buffers {
            reads: &'static [(usize, In)],
            writes: &'static [(usize, Out)],
        },
    }

    type Places = (&'static [(usize, In)], &'static [(usize, Out)]);

    impl Operand {
        fn places(&self) -> Option<Places> {
            match *self {
                Operand::Buffer { reads, writes }
                | Operand::Sized { reads, writes, .. }
                | Operand::RaceBlock { reads, writes }
                | Operand::Buffers { reads, writes } => Some((reads, writes)),
                Operand::Value(_) => None,
            }
        }
    }

    /// A block: its operands, how many float ulps apart a result may be (a transcendental's:
    /// libm's, not the x87's) and the Rust code, given the operands' values.
    pub struct Block {
        pub name: &'static str,
        pub operands: Vec<Operand>,
        pub ulps: i64,
        pub plain: fn(&[usize]),
    }

    /// The bytes every buffer operand holds past its last listed place: room for a record the
    /// Rust version lays over the buffer from a place at or below its first listed one
    /// (`physics::store::model_in_test_buffer`, `physics::load_car_specs`' records).
    pub const LAID_ROOM: usize = 0x600;

    /// The memory at a buffer operand's value (the buffer's address) as the Rust codes' test
    /// closures reach it: the one place the tests turn such a value into a pointer.
    pub fn at(address: usize) -> *mut u8 {
        assert!(address >= 0x1000, "no test buffer is at {address:#x}");
        address as *mut u8
    }

    /// An operand's memory: a buffer of the test's, or the race block at its offsets (the
    /// places live in the registry's stores, `test_support::race_places`).
    enum Mem {
        Buffer(Vec<u8>),
        Block,
        None,
    }

    impl Mem {
        fn get(&self, off: usize, len: usize) -> Vec<u8> {
            match self {
                Mem::Buffer(v) => v[off..off + len].to_vec(),
                Mem::Block => crate::test_support::race_places::read(off, len),
                Mem::None => Vec::new(),
            }
        }

        fn put(&mut self, off: usize, bytes: &[u8]) {
            match self {
                Mem::Buffer(v) => v[off..off + bytes.len()].copy_from_slice(bytes),
                Mem::Block => crate::test_support::race_places::write(off, bytes),
                Mem::None => {}
            }
        }
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

    /// Replays the vectors `name`: each block `cases` times at every control word on seeded random
    /// inputs at the places it reads, the Rust code compared at every place it writes with the
    /// recorded results (floats within the block's ulps or both NaN, whole numbers and 80-bit
    /// values bit for bit, the status words' condition bits).
    pub fn sparse(name: &str, blocks: &[Block], cases: u32, seed: u64) {
        sparse_at(name, blocks, cases, seed, &CONTROL_WORDS);
    }

    /// `sparse` at the control words `words` only.
    pub fn sparse_at(name: &str, blocks: &[Block], cases: u32, seed: u64, words: &[u16]) {
        run(Recorder::replay(name), blocks, cases, seed, words);
    }

    fn run(mut file: Recorder, blocks: &[Block], cases: u32, seed: u64, words: &[u16]) {
        // No lock ("The lock" above): the race block's places are this thread's own owners'.
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
                            let v = vec![0u8; end + 16 + LAID_ROOM];
                            args.push(v.as_ptr() as usize);
                            buffers.push(Mem::Buffer(v));
                        }
                        Operand::Sized { len, reads, writes } => {
                            let end = reads
                                .iter()
                                .map(|&(off, k)| off + width_in(k))
                                .chain(writes.iter().map(|&(off, k)| off + width_out(k)))
                                .max()
                                .unwrap_or(0);
                            let v = vec![0u8; end.max(*len) + 16 + LAID_ROOM];
                            args.push(v.as_ptr() as usize);
                            buffers.push(Mem::Buffer(v));
                        }
                        Operand::RaceBlock { reads, writes }
                        | Operand::Buffers { reads, writes } => {
                            // The block's places are reached by offset, not by an address.
                            args.push(0);
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
                let _put_back = PutBack(saved);
                // the places written, as (operand, offset, kind)
                let mut places: Vec<(usize, usize, Out)> = Vec::new();
                for (n, o) in block.operands.iter().enumerate() {
                    if let Some((_, writes)) = o.places() {
                        places.extend(writes.iter().map(|&(off, k)| (n, off, k)));
                    }
                }
                let read_out = |buffers: &Vec<Mem>| -> Vec<u8> {
                    places
                        .iter()
                        .flat_map(|&(n, off, k)| buffers[n].get(off, width_out(k)))
                        .collect()
                };
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
                                    let bytes: Vec<u8> =
                                        m.iter().flat_map(|d| d.to_le_bytes()).collect();
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
                    for (_, _, bytes) in &touched {
                        file.input(bytes);
                    }
                    let before = read_out(&buffers);
                    let expected = file.results(&before, || unreachable!("replayed"));
                    (block.plain)(&args);
                    let made = read_out(&buffers);
                    let mut at = 0;
                    for &(n, off, kind) in &places {
                        let len = width_out(kind);
                        let (a, b) = (&expected[at..at + len], &made[at..at + len]);
                        at += len;
                        let ok = match kind {
                            Out::F => {
                                let (x, y) = (
                                    u32::from_le_bytes(a.try_into().unwrap()),
                                    u32::from_le_bytes(b.try_into().unwrap()),
                                );
                                floats += 1;
                                nans += u32::from(f32::from_bits(x).is_nan());
                                close_float(x, y, block.ulps)
                            }
                            Out::I | Out::T => a == b,
                            Out::H => {
                                let (x, y) = (
                                    u16::from_le_bytes(a.try_into().unwrap()),
                                    u16::from_le_bytes(b.try_into().unwrap()),
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
        file.finish();
    }
}

// ---- single words (lane t5rest) ----

/// A case's result word, read from the file.
pub fn word(file: &mut Recorder) -> u32 {
    let bytes = file.results(&[0; 4], || unreachable!("replayed"));
    u32::from_le_bytes(bytes.try_into().unwrap())
}

/// A case's result words.
pub fn words<const N: usize>(file: &mut Recorder) -> [u32; N] {
    let bytes = file.results(&[0; 4].repeat(N), || unreachable!("replayed"));
    std::array::from_fn(|k| u32::from_le_bytes(bytes[4 * k..4 * k + 4].try_into().unwrap()))
}

/// A case's result bytes.
pub fn bytes<const N: usize>(file: &mut Recorder) -> [u8; N] {
    file.results(&[0; N], || unreachable!("replayed"))
        .try_into()
        .unwrap()
}
