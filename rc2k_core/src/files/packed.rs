//! The game's "SQSL" packed files (course data), unpacked in place.
//!
//! The file begins "SQSL", two dwords, and the count of dwords it unpacks to. Its dwords are
//! moved to the buffer's end (backwards, so a file filling the buffer stays in place); from +0x10
//! of them a stream of bits (each dword's highest first) is read, and the dwords written from the
//! buffer's start, each by a code: 0 a zero; 10 and 8 bits n, the dword n + 1 back; 1100 and 12
//! bits, likewise; 1101 and 17 bits, that number; 1110, a sign bit and 27 bits (28 bits,
//! signed); 1111 and 32 bits, that number.

use crate::Invalid;

/// "SQSL" as the file's first dword.
pub const MAGIC: u32 = 0x4C53_5153;

fn get(buffer: &[u8], at: usize) -> Result<u32, Invalid> {
    let b = buffer
        .get(at..at + 4)
        .ok_or(Invalid("a packed stream read past the buffer"))?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// The bits, each dword's highest first, a dword fetched when the last one's 32 are used.
struct Bits {
    at: usize,
    word: u32,
    left: u32,
}

impl Bits {
    fn bit(&mut self, buffer: &[u8]) -> Result<u32, Invalid> {
        self.left -= 1;
        if self.left == 0 {
            self.word = get(buffer, self.at)?;
            self.left = 32;
            self.at += 4;
        }
        let bit = self.word >> 31;
        self.word <<= 1;
        Ok(bit)
    }

    fn bits(&mut self, buffer: &[u8], mut value: u32, n: u32) -> Result<u32, Invalid> {
        for _ in 0..n {
            value = value.wrapping_mul(2) | self.bit(buffer)?;
        }
        Ok(value)
    }
}

/// Whether a file read into a buffer is packed.
pub fn is_packed(buffer: &[u8]) -> bool {
    get(buffer, 0) == Ok(MAGIC)
}

/// The file of `size` bytes at the start of `buffer` unpacked in place: the bytes written
/// from the start. Refused: a count of 0, a stream that reads or writes past the buffer or
/// refers back before its start; the buffer may be changed by then.
pub fn unpack(buffer: &mut [u8], size: usize) -> Result<usize, Invalid> {
    let room = buffer.len();
    let words = size >> 2;
    if size > room || 4 * words > room {
        return Err(Invalid("a packed file larger than its buffer"));
    }
    for k in 1..=words {
        let word = get(buffer, size - 4 * k)?;
        buffer[room - 4 * k..room - 4 * k + 4].copy_from_slice(&word.to_le_bytes());
    }
    let mut bits = Bits {
        at: room - 4 * words + 0x10,
        word: 0,
        left: 1,
    };
    let mut out = size & 3;
    let count = get(buffer, 0xC)?;
    if count == 0 {
        return Err(Invalid(
            "a packed file of no dwords: the original counts round",
        ));
    }
    for _ in 0..count {
        let back = |buffer: &[u8], out: usize, n: u32| -> Result<u32, Invalid> {
            let from = out
                .checked_sub(4 * (n as usize + 1))
                .ok_or(Invalid("a packed stream referring back past the buffer"))?;
            get(buffer, from)
        };
        let value = if bits.bit(buffer)? == 0 {
            0
        } else if bits.bit(buffer)? == 0 {
            let n = bits.bits(buffer, 0, 8)?;
            back(buffer, out, n)?
        } else if bits.bit(buffer)? == 0 {
            if bits.bit(buffer)? == 0 {
                let n = bits.bits(buffer, 0, 12)?;
                back(buffer, out, n)?
            } else {
                bits.bits(buffer, 0, 17)?
            }
        } else if bits.bit(buffer)? == 0 {
            let sign = bits.bit(buffer)?;
            bits.bits(buffer, 0u32.wrapping_sub(sign), 27)?
        } else {
            bits.bits(buffer, 0, 32)?
        };
        buffer
            .get_mut(out..out + 4)
            .ok_or(Invalid("a packed stream writing past the buffer"))?
            .copy_from_slice(&value.to_le_bytes());
        out += 4;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A packed file of raw codes: (value, bits) pairs.
    fn pack(codes: &[(u32, u32)], count: u32) -> Vec<u8> {
        let mut bits = Vec::new();
        for &(v, n) in codes {
            for k in (0..n).rev() {
                bits.push(v >> k & 1 != 0);
            }
        }
        let mut file = Vec::new();
        for d in [MAGIC, 0, 0, count] {
            file.extend(d.to_le_bytes());
        }
        for word in bits.chunks(32) {
            let mut d = 0u32;
            for (k, &b) in word.iter().enumerate() {
                d |= (b as u32) << (31 - k);
            }
            file.extend(d.to_le_bytes());
        }
        file
    }

    #[test]
    fn unpacks_every_code() {
        let file = pack(
            &[
                (0b1111, 4),
                (0xDEAD_BEEF, 32),                // 0xDEADBEEF
                (0, 1),                           // 0
                (0b10 << 8 | 1, 10),              // the dword 2 back: 0xDEADBEEF
                (0b1101 << 17 | 0x1_2345, 21),    // 0x12345
                (0b1110 << 28 | 0x0FFF_FFFF, 32), // -1
                (0b1100 << 12, 16),               // the dword 1 back: -1
            ],
            6,
        );
        let mut buffer = vec![0xAAu8; 0x100];
        buffer[..file.len()].copy_from_slice(&file);
        assert!(is_packed(&buffer));
        let n = unpack(&mut buffer, file.len()).unwrap();
        let words: Vec<u32> = buffer[..n]
            .chunks(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        assert_eq!(
            words,
            [0xDEAD_BEEF, 0, 0xDEAD_BEEF, 0x1_2345, u32::MAX, u32::MAX]
        );
    }

    #[test]
    fn refuses() {
        let mut buffer = vec![0u8; 0x40];
        let file = pack(&[(0, 1)], 0);
        buffer[..file.len()].copy_from_slice(&file);
        assert!(unpack(&mut buffer, file.len()).is_err()); // no dwords
        let file = pack(&[(0b10 << 8 | 5, 10)], 1);
        buffer[..file.len()].copy_from_slice(&file);
        assert!(unpack(&mut buffer, file.len()).is_err()); // back before the start
        let mut small = vec![0u8; 0x18];
        let file = pack(&[(0, 1)], 100);
        small[..file.len()].copy_from_slice(&file);
        assert!(unpack(&mut small, file.len()).is_err()); // writes past the buffer
    }
}
