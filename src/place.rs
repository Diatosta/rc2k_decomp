//! A place in the game's texts and data: a typed buffer and an offset into it,
//! or, for what the game's data still hands around as pointers (the installer's script
//! and pages, the name tables), a game address. Helpers take `impl Into<BytePlace>`, so a
//! typed buffer goes in as itself (`&INSTALLER_PATHS.installed`, `&TEXT_BUFFER.text`) and
//! a pointer read from the game's data as its number.

use crate::errors::BadInput;
use crate::pointers::{GamePtr, Store};
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
    /// Byte `.1` of a record's bytes (a store): reads and writes run on inside the record, as
    /// far as its end.
    Store(&'static dyn Store, usize),
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

impl core::fmt::Debug for BytePlace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BytePlace::In(_, at) => write!(f, "BytePlace::In(+{at:#x})"),
            BytePlace::At(a) => write!(f, "BytePlace::At({a:?})"),
            BytePlace::Store(_, at) => write!(f, "BytePlace::Store(+{at:#x})"),
        }
    }
}

impl crate::session::TextPlace for BytePlace {
    fn text(&self, what: &str) -> Result<Vec<u8>, BadInput> {
        BytePlace::text(*self, what)
    }

    fn write_text_at(&self, at: usize, bytes: &[u8], what: &str) -> Result<(), BadInput> {
        self.add(at as u32).write(bytes, what)
    }
}

const PAST: BadInput = BadInput("past the end of a buffer");

impl BytePlace {
    /// Byte `at` of a record's bytes.
    pub fn in_store(store: &'static dyn Store, at: usize) -> BytePlace {
        BytePlace::Store(store, at)
    }

    /// Whether both places start at the same byte.
    pub fn same_place(self, other: BytePlace) -> bool {
        self.game_ptr() == other.game_ptr()
    }

    /// The dword here.
    #[track_caller]
    pub fn read_dword(self, what: &str) -> Result<u32, BadInput> {
        Ok(u32::from_le_bytes(self.read_array::<4>(what)?))
    }

    /// `n` bytes on.
    pub fn add(self, n: u32) -> BytePlace {
        match self {
            BytePlace::Store(s, at) => BytePlace::Store(s, (at as u32).wrapping_add(n) as usize),
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
            BytePlace::Store(s, at) => {
                let mut bytes = vec![0; len];
                s.load(at, &mut bytes)?;
                Ok(bytes)
            }
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
            BytePlace::Store(s, at) => s.store(at, data),
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
            BytePlace::Store(s, at) => {
                let mut text = Vec::new();
                for k in at..s.size() {
                    let mut byte = [0];
                    s.load(k, &mut byte)?;
                    if byte[0] == 0 {
                        return Ok(text);
                    }
                    text.push(byte[0]);
                }
                Err(BadInput("a text without its end in its record"))
            }
        }
    }

    /// The bytes from here to the end of the buffer, the record or the game address's
    /// region, for the readers that walk them (`rc2k_core`'s `Bytes`).
    #[track_caller]
    pub fn bytes(self, what: &str) -> Result<PlaceBytes, BadInput> {
        Ok(match self {
            BytePlace::In(b, at) => {
                Self::inside(b, at, 0)?;
                PlaceBytes::In(b, at)
            }
            BytePlace::At(a) => PlaceBytes::At(a.bytes(what)?),
            BytePlace::Store(s, at) => PlaceBytes::Store(s, at),
        })
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
            BytePlace::Store(s, at) => GamePtr::of_store(s).offset(at as u32),
        }
    }

    /// The game address, for the script's and the game's data (pointers they keep).
    pub fn game(self) -> u32 {
        self.game_ptr().addr()
    }
}

/// The bytes from a place on, as far as its buffer, record or region goes
/// (`BytePlace::bytes`).
pub enum PlaceBytes {
    /// From byte `.1` of a typed buffer.
    In(&'static dyn Run, usize),
    /// From a game address.
    At(crate::pointers::GameBytes),
    /// From byte `.1` of a record's bytes.
    Store(&'static dyn Store, usize),
}

impl rc2k_core::strings::bytes::Bytes for PlaceBytes {
    fn byte(&self, at: usize) -> Option<u8> {
        match self {
            PlaceBytes::In(b, from) => {
                let k = from.checked_add(at)?;
                (k < b.len()).then(|| b.get(k))
            }
            PlaceBytes::At(a) => a.byte(at),
            PlaceBytes::Store(s, from) => {
                let k = from.checked_add(at)?;
                let mut byte = [0];
                (k < s.size() && s.load(k, &mut byte).is_ok()).then_some(byte[0])
            }
        }
    }
}
