//! The hasher of the run checks' hashes (`RC2K_FAN_HASH`, `RC2K_SOUND_HASH`): the standard library's
//! with fixed keys, fed every `usize` as 64 bits, so a 32-bit and a 64-bit build of the same commands
//! log the same hash (a slice's or array's length is hashed as a `usize`).

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[derive(Default)]
pub struct CheckHasher(DefaultHasher);

impl Hasher for CheckHasher {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    fn write_usize(&mut self, n: usize) {
        self.0.write_u64(n as u64);
    }

    fn write_isize(&mut self, n: isize) {
        self.0.write_i64(n as i64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::Hash;

    #[test]
    fn a_length_is_hashed_as_64_bits() {
        let mut a = CheckHasher::default();
        [1u32, 2, 3].hash(&mut a);
        let mut b = CheckHasher::default();
        3u64.hash(&mut b);
        1u32.hash(&mut b);
        2u32.hash(&mut b);
        3u32.hash(&mut b);
        assert_eq!(a.finish(), b.finish());
    }
}
