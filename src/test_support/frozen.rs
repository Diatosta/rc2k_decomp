//! Frozen results: what a test's cases gave, kept as one hash per named case set instead of
//! a reference to compute them again. The comparisons with the original's instructions
//! (the x87 reference modules, 2026-11-13) were run one last time and passed; each set's
//! results were then hashed, and the tests now check that the same inputs still give
//! results with the same hash.
//!
//! A result goes in as its bits, little-endian, a float's (or an 80-bit value's) NaN as one
//! NaN: which NaN an operation gives is not fixed in Rust (the optimiser may fold it), and
//! the comparisons the hashes stand for counted any NaN as any other.
//!
//! To make a table again (after a change meant to change the results), run the tests with
//! `RC2K_FREEZE_OUT=<file>`: each set's hash is appended to the file as a table line
//! (`("name", 0x...),`) and nothing is checked.

use std::collections::BTreeMap;

/// The FNV-1a hash being made.
#[derive(Clone, Copy)]
pub struct Digest(u64);

impl Default for Digest {
    fn default() -> Self {
        Digest(0xCBF2_9CE4_8422_2325)
    }
}

impl Digest {
    pub fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3);
        }
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

/// A result as the hash takes it.
pub trait Canon {
    fn feed(&self, d: &mut Digest);
}

/// A float's bits (the x87 helpers give their float results as bits).
impl Canon for u32 {
    fn feed(&self, d: &mut Digest) {
        let bits = if f32::from_bits(*self).is_nan() {
            0x7FC0_0000
        } else {
            *self
        };
        d.bytes(&bits.to_le_bytes());
    }
}

impl Canon for f32 {
    fn feed(&self, d: &mut Digest) {
        self.to_bits().feed(d);
    }
}

impl Canon for u64 {
    /// A double's bits.
    fn feed(&self, d: &mut Digest) {
        let bits = if f64::from_bits(*self).is_nan() {
            0x7FF8_0000_0000_0000
        } else {
            *self
        };
        d.bytes(&bits.to_le_bytes());
    }
}

impl Canon for f64 {
    fn feed(&self, d: &mut Digest) {
        self.to_bits().feed(d);
    }
}

macro_rules! whole {
    ($($t:ty),*) => {$(
        impl Canon for $t {
            fn feed(&self, d: &mut Digest) {
                d.bytes(&self.to_le_bytes());
            }
        }
    )*};
}
whole!(i32, u16, u8, i64, usize);

impl Canon for bool {
    fn feed(&self, d: &mut Digest) {
        d.bytes(&[u8::from(*self)]);
    }
}

/// An 80-bit value: its ten bytes, a NaN as one NaN.
impl Canon for [u8; 10] {
    fn feed(&self, d: &mut Digest) {
        let mantissa = u64::from_le_bytes(self[..8].try_into().unwrap());
        let nan = u16::from_le_bytes([self[8], self[9]]) & 0x7FFF == 0x7FFF && mantissa << 1 != 0;
        if nan {
            d.bytes(&[0, 0, 0, 0, 0, 0, 0, 0xC0, 0xFF, 0x7F]);
        } else {
            d.bytes(self);
        }
    }
}

/// What an array may hold: one result each (a byte array is an 80-bit value, not bytes).
pub trait Element: Canon {}
impl Element for u32 {}
impl Element for i32 {}
impl Element for u16 {}
impl Element for f32 {}
impl Element for u64 {}
impl Element for bool {}
impl Element for [u8; 10] {}
impl<T: Element, const N: usize> Element for [T; N] {}

impl<T: Element, const N: usize> Canon for [T; N] {
    fn feed(&self, d: &mut Digest) {
        for x in self {
            x.feed(d);
        }
    }
}

impl<T: Canon> Canon for Vec<T> {
    fn feed(&self, d: &mut Digest) {
        d.bytes(&(self.len() as u32).to_le_bytes());
        for x in self {
            x.feed(d);
        }
    }
}

impl<T: Canon> Canon for Option<T> {
    fn feed(&self, d: &mut Digest) {
        match self {
            None => d.bytes(&[0]),
            Some(x) => {
                d.bytes(&[1]);
                x.feed(d);
            }
        }
    }
}

impl Canon for () {
    fn feed(&self, _: &mut Digest) {}
}

impl<A: Canon, B: Canon> Canon for (A, B) {
    fn feed(&self, d: &mut Digest) {
        self.0.feed(d);
        self.1.feed(d);
    }
}

impl<A: Canon, B: Canon, C: Canon> Canon for (A, B, C) {
    fn feed(&self, d: &mut Digest) {
        self.0.feed(d);
        self.1.feed(d);
        self.2.feed(d);
    }
}

impl<A: Canon, B: Canon, C: Canon, D: Canon> Canon for (A, B, C, D) {
    fn feed(&self, d: &mut Digest) {
        self.0.feed(d);
        self.1.feed(d);
        self.2.feed(d);
        self.3.feed(d);
    }
}

/// One test's case sets, each hashed as its cases' results come.
#[derive(Default)]
pub struct Digests {
    sets: BTreeMap<String, Digest>,
}

impl Digests {
    pub fn new() -> Self {
        Self::default()
    }

    /// A case's result, into the hash of the set `name`.
    pub fn feed<T: Canon + ?Sized>(&mut self, name: &str, result: &T) {
        result.feed(self.sets.entry(name.to_string()).or_default());
    }

    /// Checks every set against `table`, its names prefixed `scope::`: each set fed must
    /// be in the table with the same hash, and each table entry of the scope must have been
    /// fed. Lists every difference.
    pub fn check(&self, scope: &str, table: &[(&str, u64)]) {
        let prefix = format!("{scope}::");
        if let Some(out) = std::env::var_os("RC2K_FREEZE_OUT") {
            let lines: String = self
                .sets
                .iter()
                .map(|(name, d)| format!("    (\"{prefix}{name}\", {:#018x}),\n", d.value()))
                .collect();
            append(&out, &lines);
            return;
        }
        let mut wrong = Vec::new();
        for (name, d) in &self.sets {
            let full = format!("{prefix}{name}");
            match table.iter().find(|(n, _)| *n == full) {
                None => wrong.push(format!("{full}: not in the table ({:#018x})", d.value())),
                Some((_, want)) if *want != d.value() => wrong.push(format!(
                    "{full}: the results' hash {:#018x}, frozen {want:#018x}",
                    d.value()
                )),
                Some(_) => {}
            }
        }
        for (name, _) in table.iter().filter(|(n, _)| n.starts_with(&prefix)) {
            if !self.sets.contains_key(&name[prefix.len()..]) {
                wrong.push(format!("{name}: frozen but not run"));
            }
        }
        assert!(
            !self.sets.is_empty() || !table.is_empty(),
            "{scope}: nothing ran"
        );
        assert!(wrong.is_empty(), "{scope}:\n{}", wrong.join("\n"));
    }
}

/// One set's hash against its entry `scope::name` in `table` (a test of one set in a scope
/// that other tests share: the scope's other entries are theirs).
pub fn check_one(scope: &str, name: &str, digest: Digest, table: &[(&str, u64)]) {
    let full = format!("{scope}::{name}");
    if let Some(out) = std::env::var_os("RC2K_FREEZE_OUT") {
        append(
            &out,
            &format!("    (\"{full}\", {:#018x}),\n", digest.value()),
        );
        return;
    }
    match table.iter().find(|(n, _)| *n == full) {
        None => panic!("{full}: not in the table ({:#018x})", digest.value()),
        Some((_, want)) => assert_eq!(
            *want,
            digest.value(),
            "{full}: the results' hash {:#018x}, frozen {want:#018x}",
            digest.value()
        ),
    }
}

/// Appends `lines` to the file `out` (`RC2K_FREEZE_OUT`).
fn append(out: &std::ffi::OsStr, lines: &str) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out)
        .expect("RC2K_FREEZE_OUT");
    file.write_all(lines.as_bytes()).expect("RC2K_FREEZE_OUT");
}
