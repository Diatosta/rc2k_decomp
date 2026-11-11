//! A place in the game's texts and data: a typed buffer and an offset into it,
//! or, for what the game's data still hands around as pointers (the installer's script
//! and pages, the name tables), a game address. Helpers take `impl Into<BytePlace>`, so a
//! typed buffer goes in as itself (`&INSTALLER_PATHS.installed`, `&TEXT_BUFFER.text`) and
//! a pointer read from the game's data as its number.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::session::{Bytes, PlainBytes};

/// A run of bytes read and written in place.
pub trait Run {
    fn len(&self) -> usize;
    fn get(&self, n: usize) -> u8;
    fn set(&self, n: usize, v: u8);
}

impl<const N: usize> Run for Bytes<N> {
    fn len(&self) -> usize {
        N
    }
    fn get(&self, n: usize) -> u8 {
        Bytes::get(self, n)
    }
    fn set(&self, n: usize, v: u8) {
        Bytes::set(self, n, v);
    }
}

impl<const N: usize> Run for PlainBytes<N> {
    fn len(&self) -> usize {
        N
    }
    fn get(&self, n: usize) -> u8 {
        PlainBytes::get(self, n)
    }
    fn set(&self, n: usize, v: u8) {
        PlainBytes::set(self, n, v);
    }
}

#[derive(Clone, Copy)]
pub enum BytePlace {
    /// Byte `.1` of a typed buffer.
    In(&'static dyn Run, usize),
    /// A game address (a pointer from the game's data), or a bounded place in a store.
    At(GamePtr),
}

impl From<u32> for BytePlace {
    fn from(at: u32) -> Self {
        BytePlace::At(GamePtr::new(at))
    }
}

impl From<GamePtr> for BytePlace {
    fn from(at: GamePtr) -> Self {
        BytePlace::At(at)
    }
}

impl<const N: usize> From<&'static Bytes<N>> for BytePlace {
    fn from(b: &'static Bytes<N>) -> Self {
        BytePlace::In(b, 0)
    }
}

impl<const N: usize> From<&'static PlainBytes<N>> for BytePlace {
    fn from(b: &'static PlainBytes<N>) -> Self {
        BytePlace::In(b, 0)
    }
}

/// A table of dwords read as bytes (the replay viewer's list, which the list menu code reads
/// as a header and items).
impl<const N: usize> From<&'static [crate::session::Dword; N]> for BytePlace {
    fn from(cells: &'static [crate::session::Dword; N]) -> Self {
        BytePlace::At(GamePtr::to(cells))
    }
}

const PAST: BadInput = BadInput("past the end of a buffer");

impl BytePlace {
    /// `n` bytes on.
    pub fn add(self, n: u32) -> BytePlace {
        match self {
            BytePlace::In(b, at) => BytePlace::In(b, at.wrapping_add(n as usize)),
            BytePlace::At(a) => BytePlace::At(a.offset(n)),
        }
    }

    /// The place's bytes `len` long, if inside its buffer (`In`); a refusal is logged with
    /// its caller (the text alone does not say which reader it was).
    #[track_caller]
    fn inside(b: &dyn Run, at: usize, len: usize) -> Result<(), BadInput> {
        match at.checked_add(len) {
            Some(end) if end <= b.len() => Ok(()),
            _ => {
                crate::framework::log(&format!(
                    "place: {len} bytes at {at} of a buffer of {}, from {}",
                    b.len(),
                    std::panic::Location::caller()
                ));
                Err(PAST)
            }
        }
    }

    /// `len` bytes from here.
    #[track_caller]
    pub fn read(self, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        match self {
            BytePlace::In(b, at) => {
                Self::inside(b, at, len)?;
                Ok((0..len).map(|k| b.get(at + k)).collect())
            }
            BytePlace::At(a) => a.read_bytes(len, what),
        }
    }

    /// `N` bytes from here, as an array.
    #[track_caller]
    pub fn read_array<const N: usize>(self, what: &str) -> Result<[u8; N], BadInput> {
        let bytes = self.read(N, what)?;
        Ok(std::array::from_fn(|i| bytes[i]))
    }

    /// `data` written from here.
    #[track_caller]
    pub fn write(self, data: &[u8], what: &str) -> Result<(), BadInput> {
        match self {
            BytePlace::In(b, at) => {
                Self::inside(b, at, data.len())?;
                for (k, &v) in data.iter().enumerate() {
                    b.set(at + k, v);
                }
                Ok(())
            }
            BytePlace::At(a) => a.write_bytes(data, what),
        }
    }

    /// The string from here, without its NUL (to the buffer's end at most for `In`).
    #[track_caller]
    pub fn text(self, what: &str) -> Result<Vec<u8>, BadInput> {
        match self {
            BytePlace::In(b, at) => {
                Self::inside(b, at, 0)?;
                let end = (at..b.len())
                    .find(|&k| b.get(k) == 0)
                    .ok_or(BadInput("a text without its end in its buffer"))?;
                Ok((at..end).map(|k| b.get(k)).collect())
            }
            BytePlace::At(a) => crate::files::cstring_at(a, what),
        }
    }

    /// The place as a checked game pointer: bounded by its buffer (`In`, which must hold
    /// at least its first byte), or a game address's region (`At`).
    pub fn game_ptr(self) -> GamePtr {
        match self {
            BytePlace::In(b, at) => {
                Self::inside(b, at, 1).unwrap_or_else(|e| crate::fatal::raise::end("place", e));
                GamePtr::to(b).offset(at as u32)
            }
            BytePlace::At(a) => a,
        }
    }

    /// The game address, for the script's and the game's data (pointers they keep).
    pub fn game(self) -> u32 {
        self.game_ptr().addr()
    }
}
