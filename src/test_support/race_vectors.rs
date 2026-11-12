//! The outputs of the original's assembly blocks that the race tests compare the plain
//! versions with, recorded once on i686 (`data/vectors/<name>.bin`): a test draws its inputs
//! from a seeded generator, so the outputs alone, in order, are the vectors.
//!
//! A test opens its vectors, hands each step's output words to `check` and calls `finish`.
//! The first `FULL` steps are kept whole (compared word by word with the test's own
//! sameness, the failing step named); every step also goes into a hash per `CHUNK` steps
//! (a chunk named when it differs). Tests whose sameness allows a difference
//! (`Kind::Close`) keep every step whole and no hashes.
//!
//! With `RC2K_RECORD_VECTORS` set, `finish` writes the file from what the test saw: run
//! so only where the words come from the original's blocks, or the test checks itself.
//!
//! The file: `RC2KVEC1`, then little-endian dwords: the steps, the words a step, the steps
//! kept whole, their words, the chunks, and each chunk's hash (two dwords, low first).

use super::first_values::hash;

/// The steps kept whole of an exact test.
const FULL: usize = 256;
/// The steps a hash covers.
const CHUNK: usize = 256;
const MAGIC: &[u8; 8] = b"RC2KVEC1";

/// How a test compares a step's words.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Bit for bit, but for NaNs (any NaN is the same as any other): hashed past `FULL`.
    Exact,
    /// By the test's own sameness (a unit in the last place): every step kept whole.
    Close,
    /// Bit for bit (words that are no floats: counts, hashes): hashed past `FULL`.
    Bits,
}

/// A test's vectors, being checked or recorded.
pub struct Vectors {
    name: &'static str,
    kind: Kind,
    words: usize,
    record: bool,
    /// The steps' words kept whole (recorded or read).
    whole: Vec<u32>,
    hashes: Vec<u64>,
    steps: usize,
    /// The steps the file holds (when checking).
    recorded: usize,
    chunk: Vec<u8>,
}

fn path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/vectors")
        .join(format!("{name}.bin"))
}

/// A NaN's bits as one NaN's: the original's NaNs and the plain versions' may differ in
/// their payload, which no test compares.
fn canonical(word: u32) -> u32 {
    if f32::from_bits(word).is_nan() {
        0x7FC0_0000
    } else {
        word
    }
}

fn dwords(bytes: &[u8]) -> impl Iterator<Item = u32> + '_ {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|&b| u32::from_le_bytes(b))
}

impl Vectors {
    /// The vectors `name`, of `words` words a step.
    pub fn open(name: &'static str, kind: Kind, words: usize) -> Vectors {
        let record = std::env::var_os("RC2K_RECORD_VECTORS").is_some();
        let mut v = Vectors {
            name,
            kind,
            words,
            record,
            whole: Vec::new(),
            hashes: Vec::new(),
            steps: 0,
            recorded: 0,
            chunk: Vec::new(),
        };
        if record {
            return v;
        }
        let bytes = std::fs::read(path(name))
            .unwrap_or_else(|e| panic!("{name}: no recorded vectors ({e})"));
        assert_eq!(&bytes[..8], MAGIC, "{name}: not a vectors file");
        let mut d = dwords(&bytes[8..]);
        let mut next = || d.next().expect("a vectors file's dword") as usize;
        v.recorded = next();
        assert_eq!(next(), words, "{name}: the words a step");
        let full = next();
        v.whole = (0..full * words).map(|_| next() as u32).collect();
        let chunks = next();
        v.hashes = (0..chunks)
            .map(|_| next() as u64 | ((next() as u64) << 32))
            .collect();
        v
    }

    /// The steps kept whole.
    fn full(&self) -> usize {
        match self.kind {
            Kind::Exact | Kind::Bits => FULL,
            Kind::Close => usize::MAX,
        }
    }

    fn close_chunk(&mut self) {
        let h = hash(&self.chunk);
        let n = self.hashes.len();
        let k = self.steps.div_ceil(CHUNK) - 1;
        if self.record {
            self.hashes.push(h);
        } else {
            assert!(
                k < n && self.hashes[k] == h,
                "{}: steps {} .. {} differ from the recorded ones",
                self.name,
                k * CHUNK,
                self.steps
            );
        }
        self.chunk.clear();
    }

    /// Step `n`'s output words `got`, the same as the recorded ones by `same`; `what` names
    /// the step's inputs when they are not.
    pub fn check(
        &mut self,
        got: &[u32],
        same: impl Fn(u32, u32) -> bool,
        what: impl FnOnce() -> String,
    ) {
        assert_eq!(got.len(), self.words, "{}: the words a step", self.name);
        let n = self.steps;
        if n < self.full() {
            if self.record {
                self.whole.extend_from_slice(got);
            } else {
                let want = self
                    .whole
                    .get(n * self.words..(n + 1) * self.words)
                    .unwrap_or_else(|| panic!("{}: more steps than recorded", self.name));
                let all = want.iter().zip(got).all(|(&w, &g)| same(w, g));
                assert!(
                    all,
                    "{} step {n}: {got:x?} where the original gave {want:x?} ({})",
                    self.name,
                    what()
                );
            }
        }
        self.steps += 1;
        if self.kind != Kind::Close {
            for &w in got {
                let w = if self.kind == Kind::Exact {
                    canonical(w)
                } else {
                    w
                };
                self.chunk.extend_from_slice(&w.to_le_bytes());
            }
            if self.steps.is_multiple_of(CHUNK) {
                self.close_chunk();
            }
        }
    }

    /// The end of the steps: every recorded step seen, or the file written.
    pub fn finish(mut self) {
        if !self.chunk.is_empty() {
            self.close_chunk();
        }
        if !self.record {
            assert_eq!(self.steps, self.recorded, "{}: the steps", self.name);
            return;
        }
        let full = self.whole.len() / self.words;
        let mut out = MAGIC.to_vec();
        let mut put = |w: u32| out.extend_from_slice(&w.to_le_bytes());
        put(self.steps as u32);
        put(self.words as u32);
        put(full as u32);
        for &w in &self.whole {
            put(w);
        }
        put(self.hashes.len() as u32);
        for &h in &self.hashes {
            put(h as u32);
            put((h >> 32) as u32);
        }
        let p = path(self.name);
        std::fs::create_dir_all(p.parent().expect("the vectors' folder"))
            .expect("the vectors' folder made");
        std::fs::write(&p, out).unwrap_or_else(|e| panic!("{}: not written ({e})", self.name));
    }
}
