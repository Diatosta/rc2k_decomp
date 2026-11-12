//! The pixels of the screen and of a texture's levels, which the game locks to draw on: a
//! handle in the place of the address DirectDraw's `Lock` gave, and a place in them reached
//! through the handle's bytes.
//!
//! The platform registers its bytes under a handle (`SCREEN`, `level`), and the game's data
//! (`DRAW_SURFACE.pixels`, `present_pixels`, a lock's description) keeps the handle where
//! the original kept the pixels' address. A `Place` is a byte of those bytes; a read or
//! write past the bytes' end, or past the extent a drawer limited it to, is refused.
//! Bytes that are gone (the screen of an earlier mode, a level let go) are refused as
//! those of an address outside every allocation were.

use crate::cells::Plain;
use crate::errors::BadInput;
use crate::pointers::PoolBytes;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

/// The screen's pixels.
pub const SCREEN: u32 = 1;

/// The pixels of the level made at `surface` (the platform's number of it).
pub const fn level(surface: usize) -> u32 {
    2 + surface as u32
}

static HELD: Mutex<BTreeMap<u32, Weak<PoolBytes>>> = Mutex::new(BTreeMap::new());

/// `bytes` named by `handle`, for as long as they live.
pub fn register(handle: u32, bytes: &Arc<PoolBytes>) {
    HELD.lock().unwrap().insert(handle, Arc::downgrade(bytes));
}

/// The bytes `handle` names; `None` for none (0), for a handle never registered, or for
/// bytes that are gone.
pub fn bytes_of(handle: u32) -> Option<Arc<PoolBytes>> {
    HELD.lock().unwrap().get(&handle)?.upgrade()
}

/// A byte of a locked surface's pixels: the bytes, the offset from their start and the
/// offset the reachable bytes end at.
#[derive(Clone)]
pub struct Place {
    bytes: Option<Arc<PoolBytes>>,
    at: u32,
    end: u32,
}

impl std::fmt::Debug for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Place({:#x} of {:#x})", self.at, self.end)
    }
}

impl Place {
    /// The first byte of the pixels `handle` names, all of them reachable.
    pub fn start(handle: u32) -> Self {
        let bytes = bytes_of(handle);
        let end = bytes.as_ref().map_or(0, |b| b.len() as u32);
        Self { bytes, at: 0, end }
    }

    /// The same place, reaching `extent` bytes from the start at most.
    pub fn limited(mut self, extent: u32) -> Self {
        self.end = self.end.min(extent);
        self
    }

    /// The place `n` bytes on (no check until it is read).
    pub fn offset(&self, n: u32) -> Self {
        Self {
            at: self.at.wrapping_add(n),
            ..self.clone()
        }
    }

    /// This place moved `n` bytes on.
    pub fn advance(&mut self, n: u32) {
        self.at = self.at.wrapping_add(n);
    }

    pub fn is_null(&self) -> bool {
        self.bytes.is_none()
    }

    /// The bytes, when `len` from this place lie inside what is reachable.
    fn check(&self, at: u32, len: usize, what: &str) -> Result<&PoolBytes, BadInput> {
        let Some(bytes) = &self.bytes else {
            crate::framework::log(&format!(
                "pixels: {what}: {len} bytes of surface pixels that are not there"
            ));
            return Err(BadInput("a locked surface with no pixels"));
        };
        if u64::from(at) + len as u64 > u64::from(self.end) {
            crate::framework::log(&format!(
                "pixels: {what}: {len} bytes at offset {:#x}, past the {:#x} reachable",
                at, self.end
            ));
            return Err(BadInput("past the end of a field"));
        }
        Ok(bytes)
    }

    pub fn read<T: Plain>(&self, what: &str) -> Result<T, BadInput> {
        self.check(self.at, size_of::<T>(), what)?
            .read_value(self.at, what)
    }

    pub fn write<T: Plain>(&self, value: T, what: &str) -> Result<(), BadInput> {
        self.check(self.at, size_of::<T>(), what)?
            .write_value(self.at, value, what)
    }

    pub fn read_bytes(&self, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        self.check(self.at, len, what)?.read_at(self.at, len, what)
    }

    pub fn write_bytes(&self, bytes: &[u8], what: &str) -> Result<(), BadInput> {
        self.check(self.at, bytes.len(), what)?
            .write_at(self.at, bytes, what)
    }

    pub fn fill(&self, value: u8, len: usize, what: &str) -> Result<(), BadInput> {
        self.check(self.at, len, what)?
            .write_at(self.at, &vec![value; len], what)
    }

    /// `bytes` written `n` bytes on, ending the program on a refusal.
    pub fn write_bytes_or_end_at(&self, n: u32, bytes: &[u8], what: &str) {
        let at = self.at.wrapping_add(n);
        if let Err(e) = self
            .check(at, bytes.len(), what)
            .and_then(|held| held.write_at(at, bytes, what))
        {
            crate::fatal::raise::end(what, e)
        }
    }

    /// The `T` `n` bytes on, ending the program on a refusal (a pixel of a copy's loop).
    pub fn read_or_end_at<T: Plain>(&self, n: u32, what: &str) -> T {
        let at = self.at.wrapping_add(n);
        self.check(at, size_of::<T>(), what)
            .and_then(|bytes| bytes.read_value(at, what))
            .unwrap_or_else(|e| crate::fatal::raise::end(what, e))
    }

    /// `value` written `n` bytes on, ending the program on a refusal.
    pub fn write_or_end_at<T: Plain>(&self, n: u32, value: T, what: &str) {
        let at = self.at.wrapping_add(n);
        if let Err(e) = self
            .check(at, size_of::<T>(), what)
            .and_then(|bytes| bytes.write_value(at, value, what))
        {
            crate::fatal::raise::end(what, e)
        }
    }

    /// `write`, ending the program on a refusal.
    pub fn write_or_end<T: Plain>(&self, value: T, what: &str) {
        if let Err(e) = self.write(value, what) {
            crate::fatal::raise::end(what, e)
        }
    }
}
