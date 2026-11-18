//! The save code's byte layer: a writer and a reader of the numbers and runs the payload is
//! made of. The reader never reads past its data and says what was missing.

use super::CodeError;

/// Appends to a payload.
#[derive(Default)]
pub struct Writer(pub Vec<u8>);

impl Writer {
    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }

    pub fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    /// A number in seven-bit groups, the low ones first, the last without the high bit.
    pub fn varint(&mut self, mut v: u32) {
        while v >= 0x80 {
            self.0.push((v & 0x7F) as u8 | 0x80);
            v >>= 7;
        }
        self.0.push(v as u8);
    }

    /// A run of bytes with its length first.
    pub fn bytes(&mut self, b: &[u8]) {
        self.varint(b.len() as u32);
        self.0.extend_from_slice(b);
    }

    /// A run of bytes of a length both sides know.
    pub fn raw(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b);
    }
}

/// Reads a payload from its start.
pub struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, at: 0 }
    }

    pub fn raw(&mut self, n: usize) -> Result<&'a [u8], CodeError> {
        let end = self
            .at
            .checked_add(n)
            .filter(|&e| e <= self.data.len())
            .ok_or(CodeError::Damaged("the code ends too soon"))?;
        let out = &self.data[self.at..end];
        self.at = end;
        Ok(out)
    }

    pub fn u8(&mut self) -> Result<u8, CodeError> {
        Ok(self.raw(1)?[0])
    }

    pub fn u32(&mut self) -> Result<u32, CodeError> {
        Ok(u32::from_le_bytes(
            self.raw(4)?.try_into().expect("4 bytes"),
        ))
    }

    pub fn varint(&mut self) -> Result<u32, CodeError> {
        let mut v = 0u32;
        for shift in (0..35).step_by(7) {
            let b = self.u8()?;
            let part = u32::from(b & 0x7F);
            if shift == 28 && part > 0xF {
                return Err(CodeError::Damaged("a number too large"));
            }
            v |= part << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(CodeError::Damaged("a number too long"))
    }

    /// A run with its length first, at most `most` long.
    pub fn bytes(&mut self, most: usize) -> Result<&'a [u8], CodeError> {
        let n = self.varint()? as usize;
        if n > most {
            return Err(CodeError::Range("a run longer than it can be"));
        }
        self.raw(n)
    }

    /// A count of at most `most`.
    pub fn count(&mut self, most: usize, what: &'static str) -> Result<usize, CodeError> {
        let n = self.varint()? as usize;
        if n > most {
            return Err(CodeError::Range(what));
        }
        Ok(n)
    }

    /// Nothing may be left.
    pub fn finish(self) -> Result<(), CodeError> {
        if self.at == self.data.len() {
            Ok(())
        } else {
            Err(CodeError::Damaged("bytes left after the code's end"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_runs_round_trip() {
        let mut w = Writer::default();
        for v in [0, 1, 127, 128, 300, 70_000, u32::MAX] {
            w.varint(v);
        }
        w.bytes(b"abc");
        w.u32(0xDEAD_BEEF);
        let mut r = Reader::new(&w.0);
        for v in [0, 1, 127, 128, 300, 70_000, u32::MAX] {
            assert_eq!(r.varint().unwrap(), v);
        }
        assert_eq!(r.bytes(10).unwrap(), b"abc");
        assert_eq!(r.u32().unwrap(), 0xDEAD_BEEF);
        r.finish().unwrap();
    }

    #[test]
    fn a_short_or_overlong_input_is_refused() {
        assert!(Reader::new(&[0x80]).varint().is_err());
        assert!(Reader::new(&[0xFF; 6]).varint().is_err());
        assert!(
            Reader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0x1F])
                .varint()
                .is_err()
        );
        assert!(Reader::new(&[5, 1, 2]).bytes(9).is_err());
        assert!(Reader::new(&[200, 1]).bytes(9).is_err());
        assert!(Reader::new(&[1, 2]).finish().is_err());
    }
}
