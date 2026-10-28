//! A place in the installer's text and data (phase 13): a typed buffer and an offset into
//! it, or, for what the script, its pages and the name tables still hand around as
//! pointers, a game address. The installer's helpers take `impl Into<Place>`, so a typed
//! buffer goes in as itself (`&INSTALLER_PATHS.installed`) and a pointer read from the
//! game's data as its number.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::session::Bytes;

/// A run of bytes the installer reads and writes in place.
pub trait Run: Sync {
    fn len(&self) -> usize;
    fn get(&self, n: usize) -> u8;
    fn set(&self, n: usize, v: u8);
    fn as_ptr(&self) -> *mut u8;
}

impl<const N: usize> Run for Bytes<N> {
    fn len(&self) -> usize {
        N
    }
    fn get(&self, n: usize) -> u8 {
        Bytes::get(self, n)
    }
    fn set(&self, n: usize, v: u8) {
        Bytes::set(self, n, v)
    }
    fn as_ptr(&self) -> *mut u8 {
        Bytes::as_ptr(self)
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

const PAST: BadInput = BadInput("past the end of an installer buffer");

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
                    .ok_or(BadInput("an installer text without its end"))?;
                Ok((at..end).map(|k| b.get(k)).collect())
            }
            Place::At(a) => crate::files::cstring(GamePtr::new(a), what),
        }
    }

    /// The host pointer for the file calls (checked to lie in its buffer or region).
    pub fn ptr(self) -> *mut u8 {
        match self {
            Place::In(b, at) => {
                Self::inside(b, at, 1).unwrap_or_else(|e| crate::install::installer::refuse(e));
                // SAFETY: `at` is inside the buffer (checked above).
                unsafe { b.as_ptr().add(at) }
            }
            Place::At(a) => crate::pointers::host(a),
        }
    }

    /// The game address, for the script's and the game's data (pointers they keep).
    pub fn game(self) -> u32 {
        match self {
            Place::In(..) => crate::pointers::game_address(self.ptr()),
            Place::At(a) => a,
        }
    }
}
