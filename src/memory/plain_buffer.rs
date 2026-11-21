//! `PlainBuffer<N>`: a buffer of bytes only the game's thread reaches, held as a plain `[u8; N]`
//! behind a `RefCell` (no cells, no raw access).
//!
//! Every method borrows for its own body and hands out copies, never a reference into the
//! buffer, so a caller cannot hold a borrow across another call on the same buffer: the borrow
//! traps of the plain stores (a borrow in a `match` head, an argument that lives for the whole
//! call) do not exist here. A read or write past the end is refused (`BadInput`) and logged.

use crate::errors::BadInput;
use crate::memory::session::{ByteBuffer, TextPlace};
use std::cell::RefCell;

pub struct PlainBuffer<const N: usize>(RefCell<[u8; N]>);

impl<const N: usize> PlainBuffer<N> {
    pub const fn new(value: [u8; N]) -> Self {
        Self(RefCell::new(value))
    }

    /// The byte at `index`.
    pub fn get(&self, index: usize) -> u8 {
        self.0.borrow()[index]
    }

    pub fn set(&self, index: usize, value: u8) {
        self.0.borrow_mut()[index] = value;
    }

    /// The little-endian dword at byte `offset` (any alignment).
    pub fn get_u32(&self, offset: usize) -> u32 {
        let all = self.0.borrow();
        u32::from_le_bytes(std::array::from_fn(|i| all[offset + i]))
    }

    pub fn set_u32(&self, offset: usize, value: u32) {
        self.0.borrow_mut()[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// All of it.
    pub fn read(&self) -> [u8; N] {
        *self.0.borrow()
    }

    /// The bytes up to the first NUL (or all of them): a text kept in the buffer.
    pub fn text(&self) -> Vec<u8> {
        let all = self.0.borrow();
        let n = all.iter().position(|&c| c == 0).unwrap_or(N);
        all[..n].to_vec()
    }

    pub fn write(&self, value: &[u8; N]) {
        *self.0.borrow_mut() = *value;
    }

    /// `len` bytes from `offset` (`what` names them in a refusal past the end).
    pub fn read_at(&self, offset: usize, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        Self::range(offset, len, what)?;
        Ok(self.0.borrow()[offset..offset + len].to_vec())
    }

    /// The byte at `offset`, as `read_at(offset, 1, what)` gives it, without the allocation.
    pub fn byte_at(&self, offset: usize, what: &str) -> Result<u8, BadInput> {
        Self::range(offset, 1, what)?;
        Ok(self.0.borrow()[offset])
    }

    /// `data` written from `offset`.
    pub fn write_at(&self, offset: usize, data: &[u8], what: &str) -> Result<(), BadInput> {
        Self::range(offset, data.len(), what)?;
        self.0.borrow_mut()[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    fn range(offset: usize, len: usize, what: &str) -> Result<(), BadInput> {
        match offset.checked_add(len) {
            Some(end) if end <= N => Ok(()),
            _ => {
                crate::memory::host_hooks::log(&format!(
                    "session: {what}: {len} bytes at {offset:#x} of a {N:#x}-byte buffer"
                ));
                Err(BadInput("past the end of a buffer"))
            }
        }
    }

    /// The text that begins `from` bytes in.
    pub fn text_from(&self, from: usize) -> crate::memory::session::TextFrom<'_, Self> {
        crate::memory::session::TextFrom::new(self, from)
    }
}

impl<const N: usize> ByteBuffer for PlainBuffer<N> {
    fn read_vec(&self) -> Vec<u8> {
        self.0.borrow().to_vec()
    }
    fn get_byte(&self, index: usize) -> u8 {
        self.get(index)
    }
    fn get_u32_at(&self, offset: usize) -> u32 {
        self.get_u32(offset)
    }
    fn read_bytes_at(&self, offset: usize, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        self.read_at(offset, len, what)
    }
    fn write_bytes_at(&self, offset: usize, data: &[u8], what: &str) -> Result<(), BadInput> {
        self.write_at(offset, data, what)
    }
}

impl<const N: usize> TextPlace for PlainBuffer<N> {
    fn text(&self, _what: &str) -> Result<Vec<u8>, BadInput> {
        let all = self.0.borrow();
        match all.iter().position(|&c| c == 0) {
            Some(n) => Ok(all[..n].to_vec()),
            None => Err(BadInput("a text with no end in its buffer")),
        }
    }

    fn write_text_at(&self, at: usize, bytes: &[u8], what: &str) -> Result<(), BadInput> {
        self.write_at(at, bytes, what)
    }
}

/// The buffer's bytes as a reader: none past its end.
impl<const N: usize> rc2k_core::strings::bytes::Bytes for PlainBuffer<N> {
    fn byte(&self, at: usize) -> Option<u8> {
        self.0.borrow().get(at).copied()
    }
    fn copy_run(&self, at: usize, out: &mut [u8]) -> Option<()> {
        let end = at.checked_add(out.len())?;
        out.copy_from_slice(self.0.borrow().get(at..end)?);
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_writes_stay_inside_the_buffer() {
        let b = PlainBuffer::<8>::new([0xAA; 8]);
        b.write_at(2, b"hi\0", "test").unwrap();
        assert_eq!(b.text(), vec![0xAA, 0xAA, b'h', b'i']);
        assert_eq!(TextPlace::text(&b, "t"), Ok(vec![0xAA, 0xAA, b'h', b'i']));
        let unended = PlainBuffer::<2>::new([1, 2]);
        assert!(TextPlace::text(&unended, "t").is_err());
        b.set_u32(4, 0x0403_0201);
        assert_eq!(b.get_u32(4), 0x0403_0201);
        assert_eq!(b.get(5), 2);
        assert!(b.write_at(6, &[0; 3], "test").is_err());
        assert!(b.read_at(usize::MAX, 2, "test").is_err());
        assert_eq!(b.byte_at(7, "test"), Ok(4));
        assert!(b.byte_at(8, "test").is_err());
        let held = PlainBuffer::<6>::new(*b"ab\0cd\0");
        assert_eq!(held.text_from(3).text("t").unwrap(), b"cd");
    }
}
