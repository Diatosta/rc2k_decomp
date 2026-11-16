//! The Indeo 4 stream's bits: a reader that takes them from each byte's low end first, and
//! the Huffman code tables built from the stream's short descriptions of them.
//!
//! Follows FFmpeg's little-endian bit reader (`libavcodec/get_bits.h`) and the code-book
//! construction of `libavcodec/ivi.c` (Copyright (c) 2009-2011 Maxim Poliakovski,
//! LGPL-2.1-or-later).

use super::Refused;

/// The longest code a code book may hold, in bits; a table has an entry for every value of
/// that many bits.
pub(super) const CODE_BITS: u32 = 13;

/// A packet read bit by bit, low bits of each byte first. Bits past the end read as 0 and
/// the place stops a byte past the end, so a damaged packet can neither run away nor read
/// outside the packet.
pub(super) struct Bits<'a> {
    bytes: &'a [u8],
    /// The next bit's number.
    at: usize,
}

impl<'a> Bits<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    /// How many bits have been taken.
    pub(super) fn taken(&self) -> usize {
        self.at
    }

    /// Bits left before the packet's end (negative once past it).
    pub(super) fn left(&self) -> i64 {
        (self.bytes.len() * 8) as i64 - self.at as i64
    }

    /// The furthest place: a byte past the packet's end.
    fn limit(&self) -> usize {
        self.bytes.len() * 8 + 8
    }

    /// The next `count` bits (at most 32) without taking them.
    pub(super) fn peek(&self, count: u32) -> u32 {
        debug_assert!(count <= 32);
        let first = self.at >> 3;
        let window = match self.bytes.get(first..first + 8) {
            Some(eight) => u64::from_le_bytes(eight.try_into().unwrap_or([0; 8])),
            None => {
                let mut eight = [0u8; 8];
                let rest = self.bytes.get(first..).unwrap_or(&[]);
                eight[..rest.len()].copy_from_slice(rest);
                u64::from_le_bytes(eight)
            }
        };
        ((window >> (self.at & 7)) & ((1u64 << count) - 1)) as u32
    }

    /// `count` bits passed over.
    pub(super) fn skip(&mut self, count: usize) {
        self.at = self.at.saturating_add(count).min(self.limit());
    }

    /// The next `count` bits (at most 32), taken.
    pub(super) fn take(&mut self, count: u32) -> u32 {
        let value = self.peek(count);
        self.skip(count as usize);
        value
    }

    /// The next bit, taken.
    pub(super) fn flag(&mut self) -> bool {
        self.take(1) != 0
    }

    /// On to the next whole byte.
    pub(super) fn align(&mut self) {
        self.skip(self.at.wrapping_neg() & 7);
    }

    /// The symbol of the next code in `book`, its bits taken; -1 with nothing taken where
    /// the bits are no code of the book.
    pub(super) fn symbol(&mut self, book: &CodeBook) -> i32 {
        let entry = book.entries[self.peek(CODE_BITS) as usize];
        self.skip((entry & 0xF) as usize);
        (entry >> 4) as i32 - 1
    }
}

/// A Huffman code book as its description: for each row (a row's codes begin with as many
/// 1 bits as its number, then a 0 except in the last row) the number of bits that follow.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) struct BookShape {
    pub rows: usize,
    pub extra_bits: [u8; 16],
}

impl BookShape {
    pub(super) const fn of(rows: usize, bits: &[u8]) -> Self {
        let mut extra_bits = [0u8; 16];
        let mut i = 0;
        while i < bits.len() {
            extra_bits[i] = bits[i];
            i += 1;
        }
        Self { rows, extra_bits }
    }

    /// Whether both describe the same book (only the rows in use count).
    pub(super) fn same_as(&self, other: &Self) -> bool {
        self.rows == other.rows && self.extra_bits[..self.rows] == other.extra_bits[..self.rows]
    }
}

/// The code books for macroblock values a header can name by number.
pub(super) const MACROBLOCK_BOOKS: [BookShape; 8] = [
    BookShape::of(8, &[0, 4, 5, 4, 4, 4, 6, 6]),
    BookShape::of(12, &[0, 2, 2, 3, 3, 3, 3, 5, 3, 2, 2, 2]),
    BookShape::of(12, &[0, 2, 3, 4, 3, 3, 3, 3, 4, 3, 2, 2]),
    BookShape::of(12, &[0, 3, 4, 4, 3, 3, 3, 3, 3, 2, 2, 2]),
    BookShape::of(13, &[0, 4, 4, 3, 3, 3, 3, 2, 3, 3, 2, 1, 1]),
    BookShape::of(9, &[0, 4, 4, 4, 4, 3, 3, 3, 2]),
    BookShape::of(10, &[0, 4, 4, 4, 4, 3, 3, 2, 2, 2]),
    BookShape::of(12, &[0, 4, 4, 4, 3, 3, 2, 3, 2, 2, 2, 2]),
];

/// The code books for block symbols a header can name by number.
pub(super) const BLOCK_BOOKS: [BookShape; 8] = [
    BookShape::of(10, &[1, 2, 3, 4, 4, 7, 5, 5, 4, 1]),
    BookShape::of(11, &[2, 3, 4, 4, 4, 7, 5, 4, 3, 3, 2]),
    BookShape::of(12, &[2, 4, 5, 5, 5, 5, 6, 4, 4, 3, 1, 1]),
    BookShape::of(13, &[3, 3, 4, 4, 5, 6, 6, 4, 4, 3, 2, 1, 1]),
    BookShape::of(11, &[3, 4, 4, 5, 5, 5, 6, 5, 4, 2, 2]),
    BookShape::of(13, &[3, 4, 5, 5, 5, 5, 6, 4, 3, 3, 2, 1, 1]),
    BookShape::of(13, &[3, 4, 5, 5, 5, 6, 5, 4, 3, 3, 2, 1, 1]),
    BookShape::of(9, &[3, 4, 4, 5, 5, 5, 6, 5, 5]),
];

/// A code book ready for reading: for every value the next `CODE_BITS` bits can have, the
/// symbol plus one in the high bits and the code's length in the low four (0: no code).
pub(super) struct CodeBook {
    entries: Vec<u16>,
}

impl CodeBook {
    /// The book `shape` describes; refused when a code would be longer than `CODE_BITS`.
    /// Only the first 256 codes of a longer description are kept.
    pub(super) fn build(shape: &BookShape) -> Result<Self, Refused> {
        let mut entries = vec![0u16; 1 << CODE_BITS];
        let mut symbol = 0u16;
        'rows: for row in 0..shape.rows.min(16) {
            let extra = u32::from(shape.extra_bits[row]);
            let not_last = u32::from(row != shape.rows - 1);
            let prefix = ((1u32 << row) - 1) << (extra + not_last);
            let length = row as u32 + extra + not_last;
            for low in 0..1u32 << extra {
                if symbol >= 256 {
                    break 'rows;
                }
                if length > CODE_BITS {
                    return Err(Refused::Damaged("a code book with a code too long"));
                }
                // A lone code of no bits takes one.
                let length = length.max(1);
                // The code is described high bit first and read low bit first.
                let code = (prefix | low).reverse_bits() >> (32 - length);
                for filler in 0..1u32 << (CODE_BITS - length) {
                    entries[(code | filler << length) as usize] = (symbol + 1) << 4 | length as u16;
                }
                symbol += 1;
            }
        }
        Ok(Self { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_come_low_end_first_and_zero_past_the_end() {
        let mut bits = Bits::new(&[0b1010_0101, 0xFF]);
        assert_eq!(bits.take(4), 0b0101);
        assert_eq!(bits.take(8), 0b1111_1010);
        assert_eq!(bits.left(), 4);
        assert_eq!(bits.take(8), 0b1111);
        assert_eq!(bits.left(), -4);
        assert_eq!(bits.take(32), 0);
        // the place stops a byte past the end
        assert_eq!(bits.taken(), 24);
        bits.skip(usize::MAX);
        assert_eq!(bits.taken(), 24);
    }

    #[test]
    fn aligning_goes_to_the_next_byte_only_when_inside_one() {
        let mut bits = Bits::new(&[0, 0, 0]);
        bits.align();
        assert_eq!(bits.taken(), 0);
        bits.take(3);
        bits.align();
        assert_eq!(bits.taken(), 8);
    }

    #[test]
    fn a_book_reads_its_codes() {
        // rows: "0" + 1 bit, "10" + 0 bits, "11" + 1 bit: symbols 0, 1; 2; 3, 4
        let book = CodeBook::build(&BookShape::of(3, &[1, 0, 1])).unwrap();
        // codes high bit first: 00, 01, 10, 110, 111. The stream holds, first bit first,
        // 00 10 110 111 01: symbols 0, 2, 3, 4, 1.
        let stream = [0b1011_0100u8, 0b1011];
        let mut bits = Bits::new(&stream);
        assert_eq!(bits.symbol(&book), 0);
        assert_eq!(bits.taken(), 2);
        assert_eq!(bits.symbol(&book), 2);
        assert_eq!(bits.symbol(&book), 3);
        assert_eq!(bits.symbol(&book), 4);
        assert_eq!(bits.symbol(&book), 1);
        assert_eq!(bits.taken(), 12);
    }

    #[test]
    fn every_listed_book_builds_and_all_but_one_are_whole() {
        for (number, shape) in MACROBLOCK_BOOKS.iter().chain(&BLOCK_BOOKS).enumerate() {
            let book = CodeBook::build(shape).unwrap();
            // the last block book describes 264 codes, of which 256 are kept
            let whole = book.entries.iter().all(|&e| e != 0);
            assert_eq!(whole, number != 15, "{shape:?}");
        }
    }

    #[test]
    fn a_code_too_long_is_refused_and_a_lone_code_takes_a_bit() {
        assert!(CodeBook::build(&BookShape::of(2, &[15, 15])).is_err());
        let book = CodeBook::build(&BookShape::of(1, &[0])).unwrap();
        let mut bits = Bits::new(&[0b10]);
        assert_eq!(bits.symbol(&book), 0);
        assert_eq!(bits.taken(), 1);
        // a 1 bit is no code of this book: nothing taken
        assert_eq!(bits.symbol(&book), -1);
        assert_eq!(bits.taken(), 1);
    }
}
