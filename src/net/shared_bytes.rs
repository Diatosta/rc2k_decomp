//! A fixed-size byte buffer both the game's thread and the receiving thread may reach: the bytes
//! sit behind a lock, and every method takes it for its own body only and hands back copies, so a
//! caller never holds a borrow across another call.

use crate::errors::BadInput;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// `N` bytes behind a lock.
pub struct SharedBytes<const N: usize>(Mutex<[u8; N]>);

impl<const N: usize> SharedBytes<N> {
    pub const fn new(value: [u8; N]) -> Self {
        Self(Mutex::new(value))
    }

    fn lock(&self) -> MutexGuard<'_, [u8; N]> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// All of it.
    pub fn read(&self) -> [u8; N] {
        *self.lock()
    }

    /// The bytes up to the first NUL (or all of them): a text kept in the buffer.
    pub fn text(&self) -> Vec<u8> {
        let all = self.lock();
        let n = all.iter().position(|&c| c == 0).unwrap_or(N);
        all[..n].to_vec()
    }

    /// All of it set from `value`.
    pub fn write(&self, value: &[u8; N]) {
        *self.lock() = *value;
    }

    /// `len` bytes from `offset` (`what` names them in a refusal past the end).
    pub fn read_at(&self, offset: usize, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        Self::range(offset, len, what)?;
        Ok(self.lock()[offset..offset + len].to_vec())
    }

    /// `data` written from `offset`.
    pub fn write_at(&self, offset: usize, data: &[u8], what: &str) -> Result<(), BadInput> {
        Self::range(offset, data.len(), what)?;
        self.lock()[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// This buffer's bytes and `other`'s exchanged (neither lock is held while the other is
    /// taken).
    pub fn swap_with(&self, other: &Self) {
        let mine = self.read();
        let theirs = other.read();
        self.write(&theirs);
        other.write(&mine);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Texts stop at the first NUL, ranges past the end are refused, and a swap exchanges all.
    #[test]
    fn reads_writes_and_swaps() {
        let one = SharedBytes::<8>::new(*b"ab\0cdefg");
        let two = SharedBytes::<8>::new([7; 8]);
        assert_eq!(one.text(), b"ab");
        assert!(one.read_at(6, 3, "a test").is_err());
        one.write_at(6, b"xy", "a test").unwrap();
        one.swap_with(&two);
        assert_eq!(two.read(), *b"ab\0cdexy");
        assert_eq!(one.read(), [7; 8]);
    }
}
