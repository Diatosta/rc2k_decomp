//! Reading an MPEG stream's bits (most significant first) and its variable-length codes, for the
//! MPEG-1 video and layer II audio decoders.
//!
//! A reader never fails: past the end of its bytes it reads zeros and goes on counting, and the
//! decoders ask afterwards (`Bits::overrun`) whether they read more than there was. A code table
//! is looked up a fixed number of bits at a time, longer codes through a second table.

/// A decoder's refusal: what in the stream it could not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmError(pub String);

impl std::fmt::Display for FilmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FilmError {}

/// A refusal with its reason.
pub(super) fn refuse<T>(why: impl Into<String>) -> Result<T, FilmError> {
    Err(FilmError(why.into()))
}

/// A run of bytes read as bits, most significant first.
#[derive(Clone)]
pub(super) struct Bits<'a> {
    bytes: &'a [u8],
    /// The next bit's number from the start.
    at: usize,
}

impl<'a> Bits<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    /// The bits read so far.
    pub(super) fn position(&self) -> usize {
        self.at
    }

    /// Whether more bits were read than the bytes hold.
    pub(super) fn overrun(&self) -> bool {
        self.at > self.bytes.len() * 8
    }

    /// The bits left (negative after an overrun).
    pub(super) fn left(&self) -> i64 {
        self.bytes.len() as i64 * 8 - self.at as i64
    }

    /// The next 32 bits without taking them (zeros past the end).
    #[inline]
    pub(super) fn peek32(&self) -> u32 {
        let byte = self.at >> 3;
        let word = match self.bytes.get(byte..byte + 8) {
            Some(b) => u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
            None => {
                let mut b = [0u8; 8];
                if let Some(rest) = self.bytes.get(byte..) {
                    b[..rest.len()].copy_from_slice(rest);
                }
                u64::from_be_bytes(b)
            }
        };
        ((word << (self.at & 7)) >> 32) as u32
    }

    /// The next `count` bits (1 .. 32) without taking them.
    #[inline]
    pub(super) fn peek(&self, count: u32) -> u32 {
        self.peek32() >> (32 - count)
    }

    #[inline]
    pub(super) fn skip(&mut self, count: u32) {
        self.at += count as usize;
    }

    /// The next `count` bits (0 .. 32), taken.
    #[inline]
    pub(super) fn take(&mut self, count: u32) -> u32 {
        if count == 0 {
            return 0;
        }
        let value = self.peek(count);
        self.skip(count);
        value
    }

    #[inline]
    pub(super) fn take_flag(&mut self) -> bool {
        self.take(1) != 0
    }
}

/// One place of a code table: a value and its code's length, or (a negative length) the place
/// of a second table and the bits that one is looked up by; length 0 is no code.
#[derive(Clone, Copy, Default)]
struct Entry {
    value: i16,
    length: i8,
}

/// A code still to be put in a table: its bits not yet looked up, their count, its value.
type Listed = (u32, u32, i16);

/// A table of variable-length codes.
pub(super) struct Codes {
    first_bits: u32,
    entries: Vec<Entry>,
}

impl Codes {
    /// The table of `codes` ((code, length) pairs, a pair's place in the list being its value),
    /// looked up `first_bits` at a time; no code may be longer than twice that.
    pub(super) fn new(codes: &[(u16, u8)], first_bits: u32) -> Self {
        let listed: Vec<Listed> = codes
            .iter()
            .enumerate()
            .map(|(value, &(code, length))| (code as u32, length as u32, value as i16))
            .collect();
        let mut made = Self {
            first_bits,
            entries: Vec::new(),
        };
        made.fill(&listed, first_bits);
        made
    }

    /// Adds a table of `bits` bits for `codes` and gives its place.
    fn fill(&mut self, codes: &[Listed], bits: u32) -> usize {
        let base = self.entries.len();
        self.entries.resize(base + (1 << bits), Entry::default());
        let mut longer: Vec<(u32, Vec<Listed>)> = Vec::new();
        for &(code, length, value) in codes {
            if length <= bits {
                let first = (code << (bits - length)) as usize;
                for entry in &mut self.entries[base + first..base + first + (1 << (bits - length))]
                {
                    *entry = Entry {
                        value,
                        length: length as i8,
                    };
                }
            } else {
                let rest = length - bits;
                let prefix = code >> rest;
                let tail = (code & ((1 << rest) - 1), rest, value);
                match longer.iter_mut().find(|(p, _)| *p == prefix) {
                    Some((_, list)) => list.push(tail),
                    None => longer.push((prefix, vec![tail])),
                }
            }
        }
        for (prefix, list) in longer {
            let sub_bits = list.iter().map(|c| c.1).max().unwrap_or(1).min(bits);
            let place = self.fill(&list, sub_bits);
            self.entries[base + prefix as usize] = Entry {
                value: place as i16,
                length: -(sub_bits as i8),
            };
        }
        base
    }

    /// The next code's value, taken; None for bits that are no code.
    #[inline]
    pub(super) fn read(&self, bits: &mut Bits) -> Option<usize> {
        let mut entry = self.entries[bits.peek(self.first_bits) as usize];
        while entry.length < 0 {
            bits.skip(self.first_bits);
            let place = entry.value as usize + bits.peek(-entry.length as u32) as usize;
            entry = *self.entries.get(place)?;
            if entry.length < 0 {
                // Tables here are two deep at most.
                return None;
            }
        }
        if entry.length == 0 {
            return None;
        }
        bits.skip(entry.length as u32);
        Some(entry.value as usize)
    }
}

/// Bytes from a text of 0s and 1s (anything else is passed over), filled up with zero bits.
#[cfg(test)]
pub(super) fn bytes_of(bits: &str) -> Vec<u8> {
    let bits: Vec<u8> = bits
        .bytes()
        .filter(|b| matches!(b, b'0' | b'1'))
        .map(|b| b - b'0')
        .collect();
    bits.chunks(8)
        .map(|byte| byte.iter().fold(0, |sum, bit| sum << 1 | bit) << (8 - byte.len()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_are_read_most_significant_first_and_zero_past_the_end() {
        let mut bits = Bits::new(&[0b1010_0000, 0xFF, 0x01]);
        assert_eq!(bits.take(3), 0b101);
        assert_eq!(bits.take(0), 0);
        assert_eq!(bits.take(13), 0xFF);
        assert_eq!(bits.left(), 8);
        assert_eq!(bits.take(32), 0x0100_0000);
        assert!(bits.overrun());
        assert_eq!(bits.left(), -24);
        assert_eq!(bits.position(), 48);
    }

    #[test]
    fn codes_longer_than_the_first_table_are_found() {
        // 1, 01, 001, 0001 0000 0001 (12 bits) in a table 6 bits wide.
        let codes = Codes::new(&[(1, 1), (1, 2), (1, 3), (0x101, 12)], 6);
        let mut bits = Bits::new(&[0b1010_0100, 0b0100_0000, 0b0100_0000]);
        let read: Vec<_> = (0..5).map(|_| codes.read(&mut bits)).collect();
        assert_eq!(read, [Some(0), Some(1), Some(2), Some(3), None]);
    }
}
