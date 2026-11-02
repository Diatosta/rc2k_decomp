//! A place in the game's texts and data (phase 13): a typed buffer and an offset into it,
//! or, for what the game's data still hands around as pointers (the installer's script
//! and pages, the name tables), a game address. Helpers take `impl Into<Place>`, so a
//! typed buffer goes in as itself (`&INSTALLER_PATHS.installed`, `&TEXT_BUFFER.text`) and
//! a pointer read from the game's data as its number. (The installer's first, 2026-10-28.)

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::session::Bytes;

/// A run of bytes read and written in place.
pub trait Run: Sync {
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

#[derive(Clone, Copy)]
pub enum Place {
    /// Byte `.1` of a typed buffer.
    In(&'static dyn Run, usize),
    /// A game address (a pointer from the game's data).
    At(u32),
}

impl From<u32> for Place {
    fn from(at: u32) -> Self {
        Place::At(at)
    }
}

impl<const N: usize> From<&'static Bytes<N>> for Place {
    fn from(b: &'static Bytes<N>) -> Self {
        Place::In(b, 0)
    }
}

const PAST: BadInput = BadInput("past the end of a buffer");

impl Place {
    /// `n` bytes on.
    pub fn add(self, n: u32) -> Place {
        match self {
            Place::In(b, at) => Place::In(b, at.wrapping_add(n as usize)),
            Place::At(a) => Place::At(a.wrapping_add(n)),
        }
    }

    /// The place's bytes `len` long, if inside its buffer (`In`).
    fn inside(b: &dyn Run, at: usize, len: usize) -> Result<(), BadInput> {
        match at.checked_add(len) {
            Some(end) if end <= b.len() => Ok(()),
            _ => Err(PAST),
        }
    }

    /// `len` bytes from here.
    pub fn read(self, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        match self {
            Place::In(b, at) => {
                Self::inside(b, at, len)?;
                Ok((0..len).map(|k| b.get(at + k)).collect())
            }
            Place::At(a) => GamePtr::new(a).read_bytes(len, what),
        }
    }

    /// `N` bytes from here, as an array.
    pub fn read_array<const N: usize>(self, what: &str) -> Result<[u8; N], BadInput> {
        let bytes = self.read(N, what)?;
        Ok(std::array::from_fn(|i| bytes[i]))
    }

    /// `data` written from here.
    pub fn write(self, data: &[u8], what: &str) -> Result<(), BadInput> {
        match self {
            Place::In(b, at) => {
                Self::inside(b, at, data.len())?;
                for (k, &v) in data.iter().enumerate() {
                    b.set(at + k, v);
                }
                Ok(())
            }
            Place::At(a) => GamePtr::new(a).write_bytes(data, what),
        }
    }

    /// The string from here, without its NUL (to the buffer's end at most for `In`).
    pub fn text(self, what: &str) -> Result<Vec<u8>, BadInput> {
        match self {
            Place::In(b, at) => {
                Self::inside(b, at, 0)?;
                let end = (at..b.len())
                    .find(|&k| b.get(k) == 0)
                    .ok_or(BadInput("a text without its end in its buffer"))?;
                Ok((at..end).map(|k| b.get(k)).collect())
            }
            Place::At(a) => crate::files::cstring(GamePtr::new(a), what),
        }
    }

    /// The place as a checked game pointer: bounded by its buffer (`In`, which must hold
    /// at least its first byte), or a game address's region (`At`).
    pub fn game_ptr(self) -> GamePtr {
        match self {
            Place::In(b, at) => {
                Self::inside(b, at, 1).unwrap_or_else(|e| crate::fatal::raise::end("place", e));
                GamePtr::to(b).offset(at as u32)
            }
            Place::At(a) => GamePtr::new(a),
        }
    }

    /// The game address, for the script's and the game's data (pointers they keep).
    pub fn game(self) -> u32 {
        self.game_ptr().addr()
    }
}
